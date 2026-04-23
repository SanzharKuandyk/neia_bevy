use std::time::Instant;

use hashbrown::HashMap;
use neia_shared::game::config::GameConfig;
use neia_shared::game::lobby::{LobbyConfig, LobbyState};
use neia_shared::game::state::GameState;
use neia_shared::server::DeltaData;
use neia_shared::{ServerError, invalid_state};

/// Game session — actual running game.
#[derive(Debug)]
pub struct GameSession {
    pub gcg: GameConfig,
    pub state: GameState,
    pub started_at: Instant,
}

/// Session lifecycle state machine.
#[derive(Debug)]
pub enum SessionState {
    /// Lobby phase — players can join, game not started.
    Lobby(Box<LobbyState>),
    /// Game is actively running.
    InGame(Box<GameSession>),
    /// Game finished, showing results.
    Finished {
        config: GameConfig,
        ended_at: Instant,
    },
}

impl SessionState {
    pub fn new_lobby(_config: GameConfig, lobby_config: LobbyConfig) -> Self {
        SessionState::Lobby(Box::new(LobbyState::new(lobby_config)))
    }

    /// Called every tick while in InGame. Returns deltas produced by game logic.
    pub fn tick(&mut self, _dt: f32, _current_tick: u64) -> Vec<DeltaData> {
        // Game-specific tick logic goes here (phases, rounds, timers, etc.)
        vec![]
    }

    /// Transition from Lobby to InGame.
    /// Validates lobby state, then converts LobbyPlayers into Players.
    pub fn start_game(
        &mut self,
        config: GameConfig,
        _current_tick: u64,
    ) -> Result<Vec<DeltaData>, ServerError> {
        let lobby = match self {
            SessionState::Lobby(lobby) => lobby,
            _ => return Err(invalid_state!("Cannot start game: not in lobby")),
        };

        // Validate lobby is ready
        lobby.can_start().map_err(ServerError::from)?;

        // Convert lobby players into game players
        let mut players = HashMap::new();
        for (id, lp) in lobby.players.drain() {
            players.insert(id, lp.into_player());
        }

        let game_state = GameState {
            current_tick: 0,
            players,
        };

        let session = GameSession {
            gcg: config,
            state: game_state,
            started_at: Instant::now(),
        };

        *self = SessionState::InGame(Box::new(session));

        // Return deltas (game-specific startup deltas added by caller)
        Ok(vec![])
    }

    /// Get mutable reference to lobby state, or error if not in Lobby.
    pub fn as_lobby_mut(&mut self) -> Result<&mut LobbyState, ServerError> {
        match self {
            SessionState::Lobby(lobby) => Ok(lobby),
            _ => Err(invalid_state!("Not in lobby state")),
        }
    }

    /// Get mutable reference to game session, or error if not InGame.
    pub fn as_game_mut(&mut self) -> Result<&mut GameSession, ServerError> {
        match self {
            SessionState::InGame(game) => Ok(game),
            _ => Err(invalid_state!("Not in game state")),
        }
    }
}
