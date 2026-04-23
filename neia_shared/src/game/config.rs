use serde::{Deserialize, Serialize};

/// Game-level configuration. Mutable per session via HTTP.
///
/// This contains game-rule tuning. Lobby constraints live in
/// `LobbyConfig` (part of server `Config`), not here.
///
/// Extend this struct with your game-specific fields.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GameConfig {
    /// Maximum player health.
    #[serde(default = "default_max_health")]
    pub max_health: i32,
}

fn default_max_health() -> i32 {
    100
}
