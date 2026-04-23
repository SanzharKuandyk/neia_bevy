use neia_shared::server::delta::{Delta, DeltaPriority};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerLeftGame {
    pub player_id: Uuid,
}

impl Delta for PlayerLeftGame {
    fn delta_type(&self) -> &'static str {
        "sys.player_left_game"
    }

    fn priority(&self) -> DeltaPriority {
        DeltaPriority::High
    }
}
