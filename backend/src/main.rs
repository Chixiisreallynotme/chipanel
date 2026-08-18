pub mod audit;
mod auth;
mod config;
pub mod container;
mod curseforge;
pub mod engine;
mod error;
mod minecraft;
mod models;
mod modrinth;
pub mod podman;
mod rcon;
mod routes;
mod websocket;

use axum::{
    http::{HeaderValue, Method, Uri},
    response::Json,
    routing::{any, get},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf, sync::Arc, time::SystemTime};
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::AppConfig;
use crate::container::{build_container_engine, ContainerEngine};
use crate::engine::GameEngineRegistry;
use crate::error::AppError;
use crate::websocket::WsHub;

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: u64,
    pub uptime_secs: u64,
}

static START_TIME: std::sync::OnceLock<SystemTime> = std::sync::OnceLock::new();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    START_TIME.get_or_init(SystemTime::now);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "chipanel=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Arc::new(AppConfig::load()?);
    let container_engine: Arc<dyn ContainerEngine> = Arc::new(build_container_engine(
        config.container_engine,
        config.container_socket_path.clone(),
    ));

    let rcon_handle = crate::rcon::RconActorHandle::spawn(config.clone());
    let engine_registry = Arc::new(GameEngineRegistry::new_default(
        config.clone(),
        container_engine.clone(),
        rcon_handle.clone(),
    ));

    let ws_hub = WsHub::new(config.clone(), rcon_handle.clone());
    let token_store = Arc::new(crate::auth::tokens::TokenStore::load_or_create(&config.data_dir).await);
    let user_store = Arc::new(crate::auth::users::UserStore::load_or_create(&config.data_dir, &config.admin_username, &config.admin_password_hash).await);
    let profile_store = Arc::new(crate::auth::profiles::ProfileStore::load_or_create(&config.data_dir).await);

    let metrics_store = Arc::new(crate::minecraft::metrics::MetricsStore::new());
    let tsdb_engine = Arc::new(crate::minecraft::tsdb::TsdbEngine::new(&config.data_dir)?);
    crate::minecraft::tsdb::start_tsdb_maintenance_loop(tsdb_engine.clone());

    let alert_config = Arc::new(tokio::sync::RwLock::new(
        crate::minecraft::metrics::AlertConfig::load_or_create(&config.data_dir).await,
    ));

    crate::minecraft::metrics::start_telemetry_sampler(
        metrics_store.clone(),
        tsdb_engine.clone(),
        alert_config.clone(),
        (*config).clone(),
        ws_hub.clone(),
    );

    crate::minecraft::version_watch::start_version_watch(config.data_dir.clone());

    crate::minecraft::tools::start_tools_sync_loop(config.clone());

    let scheduler_engine = Arc::new(
        crate::minecraft::scheduler::SchedulerEngine::new(config.clone(), Some(rcon_handle.clone())).await,
    );
    crate::minecraft::scheduler::start_scheduler_loop((*scheduler_engine).clone());

    let webhook_dispatcher = Arc::new(
        crate::minecraft::webhook::WebhookDispatcher::new(config.clone()).await,
    );

    let cors = if config.allowed_origins.iter().any(|o| o == "*") {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
            .allow_headers(Any)
    } else {
        let allowed_origins: Vec<HeaderValue> = config
            .allowed_origins
            .iter()
            .filter_map(|s| s.parse::<HeaderValue>().ok())
            .collect();

        CorsLayer::new()
            .allow_origin(allowed_origins)
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
            .allow_headers(Any)
    };

    let frontend_build_dir = find_frontend_build_dir();
    info!("Serving static frontend from: {:?}", frontend_build_dir);

    let app = Router::new()
        .nest("/api/auth", routes::auth_router())
        .nest("/api/server", routes::server_router())
        .nest("/api/plugins", routes::plugins_router())
        .nest("/api/modpacks", routes::modpacks_router())
        .nest("/api/players", routes::players_router())
        .nest("/api/permissions", routes::permissions_router())
        .nest("/api/worlds", routes::worlds_router())
        .nest("/api/files", routes::files_router())
        .nest("/api/metrics", routes::metrics_router())
        .nest("/api/profiles", routes::profiles_router())
        .nest("/api/tools", routes::tools_router())
        .nest("/api/logs", routes::logs_router())
        .nest("/api/diff", routes::diff_router())
        .nest("/api/maintenance", routes::database_router())
        .nest("/api/geyser", routes::geyser_router())
        .nest("/api/backups", routes::backups_router())
        .nest("/api/audit", routes::audit_router())
        .nest("/api/diagnostics", routes::diagnostics_router())
        .nest("/api/autotune", routes::autotune_router())
        .nest("/api/scheduler", routes::scheduler_router())
        .nest("/api/webhooks", routes::webhooks_router())
        .route("/api/public/resourcepack/:filename", get(routes::server::public_resourcepack_handler))
        .route("/api/health", get(health_handler))
        .route("/ws", get(routes::websocket::websocket_handler))
        // Unknown /api/* paths must 404 as JSON; without this the SPA fallback below
        // answers them with 200 text/html and the client chokes on "<!doctype".
        .route("/api/*rest", any(api_not_found_handler))
        .fallback_service(
            ServeDir::new(&frontend_build_dir).fallback(ServeFile::new(frontend_build_dir.join("index.html"))),
        )
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(Extension(ws_hub))
        .layer(Extension(rcon_handle))
        .layer(Extension(metrics_store))
        .layer(Extension(tsdb_engine))
        .layer(Extension(scheduler_engine))
        .layer(Extension(webhook_dispatcher))
        .layer(Extension(alert_config))
        .layer(Extension(token_store))
        .layer(Extension(user_store))
        .layer(Extension(profile_store))
        .layer(Extension(container_engine))
        .layer(Extension(engine_registry))
        .layer(Extension(config.modrinth_client.clone()))
        .layer(Extension(config.curseforge_client.clone()))
        .layer(Extension(config.clone()));

    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .unwrap_or_else(|_| SocketAddr::from(([127, 0, 0, 1], config.port)));

    info!("ChiPanel backend listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_handler() -> Json<HealthResponse> {
    let start = START_TIME.get().copied().unwrap_or_else(SystemTime::now);
    let uptime = SystemTime::now()
        .duration_since(start)
        .unwrap_or_default()
        .as_secs();

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp,
        uptime_secs: uptime,
    })
}

/// Catch-all for unmounted API paths - keeps the SPA fallback from returning HTML.
async fn api_not_found_handler(uri: Uri) -> AppError {
    AppError::NotFound(format!("No API route for '{}'", uri.path()))
}

fn find_frontend_build_dir() -> PathBuf {
    let candidates = [
        PathBuf::from("../frontend/build"),
        PathBuf::from("./frontend/build"),
        PathBuf::from("/app/frontend/build"),
        PathBuf::from("static"),
    ];

    for candidate in &candidates {
        if candidate.exists() && candidate.is_dir() {
            return candidate.clone();
        }
    }

    PathBuf::from("../frontend/build")
}
