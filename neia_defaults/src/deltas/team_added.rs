use neia_shared::server::delta::{Delta, DeltaPriority};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamAdded {
    pub team_id: u8,
    pub name: String,
}

impl Delta for TeamAdded {
    fn delta_type(&self) -> &'static str {
        "sys.team_added"
    }

    fn priority(&self) -> DeltaPriority {
        DeltaPriority::High
    }
}
