use neia_shared::server::delta::Delta;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerReadyChanged {
    pub player_id: Uuid,
    pub is_ready: bool,
}

impl Delta for PlayerReadyChanged {
    fn delta_type(&self) -> &'static str {
        "sys.player_ready_changed"
    }
}
