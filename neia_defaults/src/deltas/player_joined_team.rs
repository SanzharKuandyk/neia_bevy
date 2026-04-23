use neia_shared::server::delta::{Delta, DeltaPriority};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerJoinedTeam {
    pub player_id: Uuid,
    pub team_id: u8,
}

impl Delta for PlayerJoinedTeam {
    fn delta_type(&self) -> &'static str {
        "sys.player_joined_team"
    }

    fn priority(&self) -> DeltaPriority {
        DeltaPriority::High
    }
}
