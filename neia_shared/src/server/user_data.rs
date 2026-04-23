use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    pub id: Uuid,
    pub name: String,
    pub team_id: Option<u8>,
    pub role_id: Option<String>,
}
