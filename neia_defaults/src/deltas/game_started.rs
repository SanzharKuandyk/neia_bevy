use neia_shared::server::delta::{Delta, DeltaPriority};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStarted;

impl Delta for GameStarted {
    fn delta_type(&self) -> &'static str {
        "sys.game_started"
    }

    fn priority(&self) -> DeltaPriority {
        DeltaPriority::Critical
    }
}
