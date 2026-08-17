use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::AppConfig;
use crate::container::ContainerEngine;
use crate::engine::driver::GameDriver;
use crate::engine::minecraft::MinecraftDriver;
use crate::engine::palworld::PalworldDriver;
use crate::engine::valheim::ValheimDriver;
use crate::rcon::RconActorHandle;

/// Thread-safe registry managing available and active game server drivers.
#[derive(Clone)]
pub struct GameEngineRegistry {
    drivers: Arc<RwLock<HashMap<String, Arc<dyn GameDriver>>>>,
    active_driver_id: Arc<RwLock<String>>,
}

impl GameEngineRegistry {
    /// Creates a new empty registry with a specified active driver id.
    pub fn new(active_driver_id: &str) -> Self {
        Self {
            drivers: Arc::new(RwLock::new(HashMap::new())),
            active_driver_id: Arc::new(RwLock::new(active_driver_id.to_string())),
        }
    }

    /// Initializes a default registry pre-loaded with Minecraft, Palworld, and Valheim drivers.
    pub fn new_default(
        config: Arc<AppConfig>,
        container: Arc<dyn ContainerEngine>,
        rcon: RconActorHandle,
    ) -> Self {
        let mc_driver = Arc::new(MinecraftDriver::new(
            config.clone(),
            container.clone(),
            rcon,
        ));
        let pal_driver = Arc::new(PalworldDriver::new(config.clone(), container.clone()));
        let val_driver = Arc::new(ValheimDriver::new(config.clone(), container.clone()));

        let mut map = HashMap::new();
        map.insert(mc_driver.game_id().to_string(), mc_driver as Arc<dyn GameDriver>);
        map.insert(pal_driver.game_id().to_string(), pal_driver as Arc<dyn GameDriver>);
        map.insert(val_driver.game_id().to_string(), val_driver as Arc<dyn GameDriver>);

        Self {
            drivers: Arc::new(RwLock::new(map)),
            active_driver_id: Arc::new(RwLock::new(config.game_driver.clone())),
        }
    }

    /// Registers a new game driver.
    pub async fn register_driver(&self, driver: Arc<dyn GameDriver>) {
        let mut map = self.drivers.write().await;
        map.insert(driver.game_id().to_string(), driver);
    }

    /// Retrieves a game driver by its unique identifier.
    pub async fn get_driver(&self, game_id: &str) -> Option<Arc<dyn GameDriver>> {
        let map = self.drivers.read().await;
        map.get(game_id).cloned()
    }

    /// Sets the active game driver identifier.
    pub async fn set_active_driver_id(&self, game_id: &str) {
        let mut active = self.active_driver_id.write().await;
        *active = game_id.to_string();
    }

    /// Retrieves the currently active game driver, falling back to Minecraft if not found.
    pub async fn get_active_driver(&self) -> Option<Arc<dyn GameDriver>> {
        let active_id = self.active_driver_id.read().await.clone();
        let map = self.drivers.read().await;
        map.get(&active_id).cloned().or_else(|| map.get("minecraft").cloned())
    }

    /// Lists all registered game driver identifiers and display names.
    pub async fn list_drivers(&self) -> Vec<(&'static str, &'static str)> {
        let map = self.drivers.read().await;
        map.values()
            .map(|d| (d.game_id(), d.display_name()))
            .collect()
    }
}
