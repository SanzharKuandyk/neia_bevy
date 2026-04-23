use serde::{Deserialize, Serialize};

/// A team in the game lobby/session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: u8,
    pub name: String,
}

impl Team {
    pub fn new(id: u8, name: String) -> Self {
        Self { id, name }
    }
}
