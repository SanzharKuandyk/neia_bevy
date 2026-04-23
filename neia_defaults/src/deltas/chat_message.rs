use neia_shared::server::delta::{Delta, DeltaPriority};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub player_id: Uuid,
    pub message: String,
}

impl Delta for ChatMessage {
    fn delta_type(&self) -> &'static str {
        "sys.chat_message"
    }

    fn priority(&self) -> DeltaPriority {
        DeltaPriority::Low
    }
}
