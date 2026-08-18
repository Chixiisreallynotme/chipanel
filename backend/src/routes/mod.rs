pub mod audit;
pub mod auth;
pub mod autotune;
pub mod backups;
pub mod database;
pub mod diagnostics;
pub mod diff;
pub mod engine_catalog;
pub mod files;
pub mod geyser;
pub mod logs;
pub mod metrics;
pub mod modpacks;
pub mod permissions;
pub mod players;
pub mod plugins;
pub mod profiles;
pub mod server;
pub mod tools;
pub mod websocket;
pub mod worlds;

// Routers live in their own module - do NOT redefine them here, a local copy silently
// shadows the real one (that is how /api/auth/users and /api/auth/tokens went missing).
pub use audit::audit_router;
pub use auth::auth_router;
pub use autotune::autotune_router;
pub use backups::backups_router;
pub use database::database_router;
pub use diagnostics::diagnostics_router;
pub use diff::diff_router;
pub use files::files_router;
pub use geyser::geyser_router;
pub use logs::logs_router;
pub use metrics::metrics_router;
pub use modpacks::modpacks_router;
pub use permissions::permissions_router;
pub use plugins::plugins_router;
pub use profiles::profiles_router;
pub use server::server_router;
pub use tools::tools_router;
pub use worlds::worlds_router;

use axum::{
    routing::{delete, get, post},
    Router,
};

// players.rs router with pending command queue endpoints
pub fn players_router() -> Router {
    Router::new()
        .route("/", get(players::list_players_handler))
        .route("/action", post(players::player_action_handler))
        .route("/pending-commands", get(players::list_pending_commands_handler))
        .route("/pending-commands/history", get(players::list_pending_history_handler))
        .route("/pending-commands/:id", delete(players::delete_pending_command_handler))
        .route("/:uuid", get(players::get_player_handler))
        .route("/:uuid/pending-commands", get(players::list_player_pending_commands_handler))
        .route("/:uuid/inventory", get(players::get_player_inventory_handler))
        .route("/:uuid/effects", get(players::get_player_effects_handler))
        .route("/:uuid/effects/apply", post(players::apply_player_effect_handler))
        .route("/:uuid/effects/clear", post(players::clear_player_effects_handler))
        .route("/:uuid/permissions", get(players::get_player_permissions_handler))
        .route("/:uuid/permissions/group", post(players::set_player_group_handler))
}
