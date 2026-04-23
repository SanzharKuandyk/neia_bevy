use std::sync::Arc;

use neia_mod::ModCollection;
use neia_shared::game::config::GameConfig;
use neia_shared::server::registry::Registry;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::server::state::ServerState;

pub struct AppState {
    pub config: Config,
    pub game_config: RwLock<GameConfig>,
    pub loaded_mods: ModCollection,
    pub server_state: ServerState,
    pub registry: Registry,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("config", &self.config)
            .field("server_state", &self.server_state)
            .field("loaded_mods", &self.loaded_mods.loaded.len())
            .field("registry_plugins", &self.registry.plugins())
            .finish()
    }
}

impl AppState {
    pub fn new(
        config: Config,
        game_config: GameConfig,
        loaded_mods: ModCollection,
        registry: Registry,
    ) -> Arc<Self> {
        let lobby_config = config.lobby_config().clone();
        Arc::new(Self {
            config,
            game_config: RwLock::new(game_config.clone()),
            loaded_mods,
            server_state: ServerState::new(game_config, lobby_config),
            registry,
        })
    }

    pub async fn get_game_config(&self) -> GameConfig {
        self.game_config.read().await.clone()
    }

    pub async fn set_game_config(&self, new_gcg: GameConfig) {
        let mut gcg = self.game_config.write().await;
        *gcg = new_gcg;
    }

    pub fn get_server_state(&self) -> &ServerState {
        &self.server_state
    }
}
