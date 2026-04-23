use hashbrown::HashMap;
use neia_shared::game::config::GameConfig;
use neia_shared::game::lobby::LobbyConfig;
use renet::ClientId;
use tokio::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use uuid::Uuid;

use crate::server::session::SessionState;

#[derive(Default, Debug)]
pub struct OwnerInfo {
    pub cid: Option<ClientId>,
    pub player_id: Option<Uuid>,
}

#[derive(Debug)]
pub struct ServerState {
    /// To differentiate who is the host.
    pub owner_info: Mutex<OwnerInfo>,
    /// Mapping from connection (client_id) to player_id.
    pub client_map: Mutex<HashMap<ClientId, Uuid>>,
    /// Session state (Lobby -> InGame -> Finished).
    pub session: RwLock<SessionState>,
}

impl ServerState {
    pub fn new(gcg: GameConfig, lobby_config: LobbyConfig) -> Self {
        Self {
            owner_info: Mutex::new(OwnerInfo::default()),
            client_map: Mutex::new(HashMap::new()),
            session: RwLock::new(SessionState::new_lobby(gcg, lobby_config)),
        }
    }

    pub async fn get_owner_info(&self) -> MutexGuard<'_, OwnerInfo> {
        self.owner_info.lock().await
    }

    /// Get a player's UUID by their client_id.
    pub async fn get_player_id(&self, client_id: ClientId) -> Option<Uuid> {
        let client_map = self.client_map.lock().await;
        client_map.get(&client_id).copied()
    }

    pub async fn get_session_state(&self) -> RwLockReadGuard<'_, SessionState> {
        self.session.read().await
    }

    pub async fn get_session_state_mut(&self) -> RwLockWriteGuard<'_, SessionState> {
        self.session.write().await
    }
}
