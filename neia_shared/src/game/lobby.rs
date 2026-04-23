use std::fmt;
use std::time::Instant;

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{ServerError, invalid_state};

use super::player::Player;
use super::team::Team;

// ─── LobbyConfig ───────────────────────────────────────────────────

/// Server-level lobby constraints.
///
/// Lives in `Config` (not `GameConfig`) because these are infrastructure
/// decisions, not game tuning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyConfig {
    /// Enable team-based play. When false, players join without teams (FFA).
    #[serde(default = "default_true")]
    pub teams_enabled: bool,

    #[serde(default = "default_min_teams")]
    pub min_teams: usize,
    #[serde(default = "default_max_teams")]
    pub max_teams: usize,

    #[serde(default = "default_min_players_per_team")]
    pub min_players_per_team: usize,
    #[serde(default = "default_max_players_per_team")]
    pub max_players_per_team: usize,

    /// Total player cap (applies regardless of teams).
    #[serde(default = "default_max_players")]
    pub max_players: usize,

    #[serde(default = "default_true")]
    pub require_all_ready: bool,

    /// Whether spectators can watch without joining a team.
    #[serde(default = "default_true")]
    pub allow_spectators: bool,
}

impl Default for LobbyConfig {
    fn default() -> Self {
        Self {
            teams_enabled: true,
            min_teams: default_min_teams(),
            max_teams: default_max_teams(),
            min_players_per_team: default_min_players_per_team(),
            max_players_per_team: default_max_players_per_team(),
            max_players: default_max_players(),
            require_all_ready: true,
            allow_spectators: true,
        }
    }
}

fn default_true() -> bool {
    true
}
fn default_min_teams() -> usize {
    2
}
fn default_max_teams() -> usize {
    4
}
fn default_min_players_per_team() -> usize {
    1
}
fn default_max_players_per_team() -> usize {
    4
}
fn default_max_players() -> usize {
    16
}

// ─── LobbyPlayer ──────────────────────────────────────────────────

/// A player sitting in the lobby before game start.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyPlayer {
    pub id: Uuid,
    pub name: String,
    /// Team assignment. None when teams are disabled or player hasn't chosen.
    pub team_id: Option<u8>,
    pub is_ready: bool,
    /// Preferred role (e.g., "game.warrior"). Interpretation is game-specific.
    pub role_id: Option<String>,
}

impl LobbyPlayer {
    pub fn new(id: Uuid, name: String, team_id: Option<u8>, role_id: Option<String>) -> Self {
        Self {
            id,
            name,
            team_id,
            is_ready: false,
            role_id,
        }
    }

    /// Convert lobby player into a full Player for game start.
    pub fn into_player(self) -> Player {
        Player::new(self.id, self.name, self.team_id, self.role_id)
    }
}

// ─── SessionError ──────────────────────────────────────────────────

/// Errors specific to lobby / session state transitions.
#[derive(Debug, Clone)]
pub enum SessionError {
    TooManyTeams {
        current: usize,
        max: usize,
    },
    DuplicateTeamName {
        name: String,
    },
    NotEnoughTeams {
        current: usize,
        min: usize,
    },
    NotEnoughPlayers {
        current: usize,
        min: usize,
    },
    GameAlreadyStarted,
    InvalidStateTransition {
        from: String,
        to: String,
    },
    PlayerAlreadyInLobby {
        id: Uuid,
    },
    PlayerNotFound {
        id: Uuid,
    },
    TeamNotFound {
        team_id: u8,
    },
    TeamFull {
        team_id: u8,
        max: usize,
    },
    TeamTooSmall {
        team_id: u8,
        team_name: String,
        current: usize,
        min: usize,
    },
    PlayersNotReady {
        unready_count: usize,
    },
    LobbyFull {
        max: usize,
    },
    TeamsDisabled,
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyTeams { current, max } => {
                write!(f, "Cannot add team: already at max ({current}/{max})")
            }
            Self::DuplicateTeamName { name } => {
                write!(f, "Team name '{name}' already exists")
            }
            Self::NotEnoughTeams { current, min } => {
                write!(f, "Need at least {min} teams to start (have {current})")
            }
            Self::NotEnoughPlayers { current, min } => {
                write!(f, "Need at least {min} players to start (have {current})")
            }
            Self::GameAlreadyStarted => write!(f, "Cannot modify lobby: game already started"),
            Self::InvalidStateTransition { from, to } => {
                write!(f, "Invalid state transition: {from} -> {to}")
            }
            Self::PlayerAlreadyInLobby { id } => write!(f, "Player {id} already in lobby"),
            Self::PlayerNotFound { id } => write!(f, "Player {id} not found"),
            Self::TeamNotFound { team_id } => write!(f, "Team {team_id} not found"),
            Self::TeamFull { team_id, max } => {
                write!(f, "Team {team_id} is full (max {max} players)")
            }
            Self::TeamTooSmall {
                team_id,
                team_name,
                current,
                min,
            } => {
                write!(
                    f,
                    "Team {team_id} '{team_name}' has too few players ({current}/{min})"
                )
            }
            Self::PlayersNotReady { unready_count } => {
                write!(f, "{unready_count} player(s) not ready")
            }
            Self::LobbyFull { max } => write!(f, "Lobby is full (max {max} players)"),
            Self::TeamsDisabled => write!(f, "Teams are not enabled for this lobby"),
        }
    }
}

impl std::error::Error for SessionError {}

impl From<SessionError> for ServerError {
    fn from(e: SessionError) -> Self {
        invalid_state!(&e.to_string())
    }
}

// ─── LobbyState ────────────────────────────────────────────────────

/// Lobby state before game starts.
#[derive(Debug)]
pub struct LobbyState {
    pub created_at: Instant,
    pub lobby_config: LobbyConfig,
    pub teams: HashMap<u8, Team>,
    pub players: HashMap<Uuid, LobbyPlayer>,
}

impl LobbyState {
    pub fn new(lobby_config: LobbyConfig) -> Self {
        Self {
            created_at: Instant::now(),
            lobby_config,
            teams: HashMap::new(),
            players: HashMap::new(),
        }
    }

    // ── Team operations ────────────────────────────────────────────

    /// Add a new team. Returns the assigned team id.
    pub fn add_team(&mut self, name: String) -> Result<u8, SessionError> {
        if !self.lobby_config.teams_enabled {
            return Err(SessionError::TeamsDisabled);
        }
        if self.teams.len() >= self.lobby_config.max_teams {
            return Err(SessionError::TooManyTeams {
                current: self.teams.len(),
                max: self.lobby_config.max_teams,
            });
        }
        if self.teams.values().any(|t| t.name == name) {
            return Err(SessionError::DuplicateTeamName { name });
        }

        let team_id = self.teams.len() as u8;
        self.teams.insert(team_id, Team::new(team_id, name));
        Ok(team_id)
    }

    /// Remove a team by id.
    pub fn remove_team(&mut self, team_id: u8) -> Option<Team> {
        self.teams.remove(&team_id)
    }

    // ── Player operations ──────────────────────────────────────────

    /// Add a player to the lobby.
    pub fn add_player(
        &mut self,
        id: Uuid,
        team_id: Option<u8>,
        name: String,
        role_id: Option<String>,
    ) -> Result<(), SessionError> {
        if self.players.contains_key(&id) {
            return Err(SessionError::PlayerAlreadyInLobby { id });
        }

        // Total cap
        if self.players.len() >= self.lobby_config.max_players {
            return Err(SessionError::LobbyFull {
                max: self.lobby_config.max_players,
            });
        }

        // Team validation (only when teams are enabled)
        if self.lobby_config.teams_enabled {
            let tid = team_id.ok_or(SessionError::TeamNotFound { team_id: 0 })?;
            if !self.teams.contains_key(&tid) {
                return Err(SessionError::TeamNotFound { team_id: tid });
            }
            let team_count = self
                .players
                .values()
                .filter(|p| p.team_id == Some(tid))
                .count();
            if team_count >= self.lobby_config.max_players_per_team {
                return Err(SessionError::TeamFull {
                    team_id: tid,
                    max: self.lobby_config.max_players_per_team,
                });
            }
        }

        self.players
            .insert(id, LobbyPlayer::new(id, name, team_id, role_id));
        Ok(())
    }

    /// Remove a player from the lobby.
    pub fn remove_player(&mut self, player_id: Uuid) -> Option<LobbyPlayer> {
        self.players.remove(&player_id)
    }

    /// Change a player's team (validates target team exists and is not full).
    pub fn assign_player_to_team(
        &mut self,
        player_id: Uuid,
        team_id: u8,
    ) -> Result<(), SessionError> {
        if !self.lobby_config.teams_enabled {
            return Err(SessionError::TeamsDisabled);
        }
        if !self.teams.contains_key(&team_id) {
            return Err(SessionError::TeamNotFound { team_id });
        }

        let team_count = self
            .players
            .values()
            .filter(|p| p.team_id == Some(team_id) && p.id != player_id)
            .count();
        if team_count >= self.lobby_config.max_players_per_team {
            return Err(SessionError::TeamFull {
                team_id,
                max: self.lobby_config.max_players_per_team,
            });
        }

        let player = self
            .players
            .get_mut(&player_id)
            .ok_or(SessionError::PlayerNotFound { id: player_id })?;
        player.team_id = Some(team_id);
        Ok(())
    }

    /// Set player ready status.
    pub fn set_player_ready(&mut self, player_id: Uuid, ready: bool) -> Result<(), SessionError> {
        let player = self
            .players
            .get_mut(&player_id)
            .ok_or(SessionError::PlayerNotFound { id: player_id })?;
        player.is_ready = ready;
        Ok(())
    }

    /// Get all players on a specific team.
    pub fn get_team_players(&self, team_id: u8) -> Vec<&LobbyPlayer> {
        self.players
            .values()
            .filter(|p| p.team_id == Some(team_id))
            .collect()
    }

    // ── Start validation ───────────────────────────────────────────

    /// Validate whether the lobby can transition to InGame.
    pub fn can_start(&self) -> Result<(), SessionError> {
        // Ready check
        if self.lobby_config.require_all_ready {
            let unready = self.players.values().filter(|p| !p.is_ready).count();
            if unready > 0 {
                return Err(SessionError::PlayersNotReady {
                    unready_count: unready,
                });
            }
        }

        if self.lobby_config.teams_enabled {
            // Team count
            if self.teams.len() < self.lobby_config.min_teams {
                return Err(SessionError::NotEnoughTeams {
                    current: self.teams.len(),
                    min: self.lobby_config.min_teams,
                });
            }

            // Per-team player count
            for (tid, team) in &self.teams {
                let count = self.get_team_players(*tid).len();
                if count < self.lobby_config.min_players_per_team {
                    return Err(SessionError::TeamTooSmall {
                        team_id: *tid,
                        team_name: team.name.clone(),
                        current: count,
                        min: self.lobby_config.min_players_per_team,
                    });
                }
            }
        } else {
            // FFA: just need enough players overall
            let total = self.players.len();
            if total < 2 {
                return Err(SessionError::NotEnoughPlayers {
                    current: total,
                    min: 2,
                });
            }
        }

        Ok(())
    }

    pub fn team_count(&self) -> usize {
        self.teams.len()
    }

    pub fn player_count(&self) -> usize {
        self.players.len()
    }
}

// ─── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn team_lobby() -> LobbyState {
        LobbyState::new(LobbyConfig::default())
    }

    fn ffa_lobby() -> LobbyState {
        LobbyState::new(LobbyConfig {
            teams_enabled: false,
            ..Default::default()
        })
    }

    #[test]
    fn add_teams() {
        let mut lobby = team_lobby();
        let t0 = lobby.add_team("Alpha".into()).unwrap();
        let t1 = lobby.add_team("Bravo".into()).unwrap();
        assert_eq!(t0, 0);
        assert_eq!(t1, 1);
        assert_eq!(lobby.team_count(), 2);
    }

    #[test]
    fn duplicate_team_name() {
        let mut lobby = team_lobby();
        lobby.add_team("Alpha".into()).unwrap();
        assert!(matches!(
            lobby.add_team("Alpha".into()),
            Err(SessionError::DuplicateTeamName { .. })
        ));
    }

    #[test]
    fn max_teams() {
        let mut lobby = LobbyState::new(LobbyConfig {
            max_teams: 2,
            ..Default::default()
        });
        lobby.add_team("A".into()).unwrap();
        lobby.add_team("B".into()).unwrap();
        assert!(matches!(
            lobby.add_team("C".into()),
            Err(SessionError::TooManyTeams { .. })
        ));
    }

    #[test]
    fn teams_disabled_blocks_add_team() {
        let mut lobby = ffa_lobby();
        assert!(matches!(
            lobby.add_team("X".into()),
            Err(SessionError::TeamsDisabled)
        ));
    }

    #[test]
    fn can_start_team_mode() {
        let mut lobby = LobbyState::new(LobbyConfig {
            min_teams: 2,
            min_players_per_team: 1,
            ..Default::default()
        });
        lobby.add_team("A".into()).unwrap();
        lobby.add_team("B".into()).unwrap();

        let p1 = Uuid::new_v4();
        let p2 = Uuid::new_v4();
        lobby.add_player(p1, Some(0), "Alice".into(), None).unwrap();
        lobby.add_player(p2, Some(1), "Bob".into(), None).unwrap();

        // Not ready yet
        assert!(lobby.can_start().is_err());

        lobby.set_player_ready(p1, true).unwrap();
        lobby.set_player_ready(p2, true).unwrap();
        assert!(lobby.can_start().is_ok());
    }

    #[test]
    fn can_start_ffa_mode() {
        let mut lobby = ffa_lobby();
        let p1 = Uuid::new_v4();
        let p2 = Uuid::new_v4();
        lobby.add_player(p1, None, "Alice".into(), None).unwrap();
        lobby.add_player(p2, None, "Bob".into(), None).unwrap();

        lobby.set_player_ready(p1, true).unwrap();
        lobby.set_player_ready(p2, true).unwrap();
        assert!(lobby.can_start().is_ok());
    }
}
