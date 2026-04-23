use neia_shared::server::delta::{Delta, DeltaPriority};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerJoinedLobby {
    pub player_id: Uuid,
    pub name: String,
}

impl Delta for PlayerJoinedLobby {
    fn delta_type(&self) -> &'static str {
        "sys.player_joined_lobby"
    }

    fn priority(&self) -> DeltaPriority {
        DeltaPriority::High
    }
}
