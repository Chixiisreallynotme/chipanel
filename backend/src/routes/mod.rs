pub mod auth;
pub mod engine_catalog;
pub mod files;
pub mod metrics;
pub mod modpacks;
pub mod permissions;
pub mod players;
pub mod plugins;
pub mod profiles;
pub mod server;
pub mod websocket;
pub mod worlds;

// Routers live in their own module - do NOT redefine them here, a local copy silently
// shadows the real one (that is how /api/auth/users and /api/auth/tokens went missing).
pub use auth::auth_router;
pub use files::files_router;
pub use metrics::metrics_router;
pub use modpacks::modpacks_router;
pub use permissions::permissions_router;
pub use plugins::plugins_router;
pub use profiles::profiles_router;
pub use server::server_router;
pub use worlds::worlds_router;

use axum::{
    routing::{get, post},
    Router,
};

// players.rs does not define its own router; keep this one here until it does.
pub fn players_router() -> Router {
    Router::new()
        .route("/", get(players::list_players_handler))
        .route("/action", post(players::player_action_handler))
        .route("/:uuid", get(players::get_player_handler))
        .route("/:uuid/inventory", get(players::get_player_inventory_handler))
        .route("/:uuid/effects", get(players::get_player_effects_handler))
        .route("/:uuid/effects/apply", post(players::apply_player_effect_handler))
        .route("/:uuid/effects/clear", post(players::clear_player_effects_handler))
        .route("/:uuid/permissions", get(players::get_player_permissions_handler))
        .route("/:uuid/permissions/group", post(players::set_player_group_handler))
}
