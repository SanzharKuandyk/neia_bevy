use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    /// Team assignment. None when teams are disabled.
    pub team_id: Option<u8>,
    pub health: i32,
    pub max_health: i32,
    pub role_id: Option<String>,
}

impl Player {
    pub fn new(id: Uuid, name: String, team_id: Option<u8>, role_id: Option<String>) -> Self {
        Self {
            id,
            name,
            team_id,
            health: 100,
            max_health: 100,
            role_id,
        }
    }
}
