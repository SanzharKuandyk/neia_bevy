use std::path::PathBuf;

use bevy::prelude::*;
use hashbrown::HashMap;
use neia_client::ClientStatus;
use neia_config::LoadedConfigs;
use neia_mod::ModCollection;
use neia_shared::UserData;
use neia_shared::app::AppConfig;
use neia_shared::game::config::GameConfig;
use uuid::Uuid;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Screen {
    #[default]
    Title,
    Settings,
    Lobby,
    Gameplay,
}

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Pause(pub bool);

#[derive(Debug, Clone)]
pub struct LobbyPlayerView {
    pub id: Uuid,
    pub name: String,
    pub is_ready: bool,
}

#[derive(Resource, Default)]
pub struct LobbyReplica {
    pub players: HashMap<Uuid, LobbyPlayerView>,
}

#[derive(Resource, Default)]
pub struct GameSession {
    pub last_tick: u64,
    pub started: bool,
}

#[derive(Resource, Clone)]
pub struct ConfigResource {
    pub root: PathBuf,
    pub app: AppConfig,
    pub game: GameConfig,
}

impl ConfigResource {
    pub fn from_loaded(loaded: LoadedConfigs) -> Self {
        Self {
            root: loaded.root,
            app: loaded.app,
            game: loaded.game,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct LoadedModsResource(pub ModCollection);

#[derive(Resource, Clone)]
pub struct LocalProfile {
    pub user_data: UserData,
}

impl LocalProfile {
    pub fn new(name: String) -> Self {
        Self {
            user_data: UserData {
                id: Uuid::new_v4(),
                name,
                team_id: None,
                role_id: None,
            },
        }
    }
}

#[derive(Resource, Default)]
pub struct NetworkState {
    pub status: ClientStatus,
    pub connected: bool,
}
