use hashbrown::HashMap;
use uuid::Uuid;

use crate::{ServerError, player_not_found};

use super::Player;

#[derive(Debug)]
pub struct GameState {
    /// Current game tick
    pub current_tick: u64,

    /// All active players
    pub players: HashMap<Uuid, Player>,
}

impl GameState {
    pub fn has_player(&self, player_id: Uuid) -> bool {
        self.players.contains_key(&player_id)
    }

    pub fn get_player(&self, player_id: Uuid) -> Result<&Player, ServerError> {
        self.players
            .get(&player_id)
            .ok_or(player_not_found!(player_id))
    }

    pub fn get_player_mut(&mut self, player_id: Uuid) -> Result<&mut Player, ServerError> {
        self.players
            .get_mut(&player_id)
            .ok_or(player_not_found!(player_id))
    }

    pub fn add_player(&mut self, player: Player) {
        self.players.insert(player.id, player);
    }

    pub fn remove_player(&mut self, player_id: Uuid) -> Result<(), ServerError> {
        if self.players.remove(&player_id).is_some() {
            Ok(())
        } else {
            Err(player_not_found!(player_id))
        }
    }
}
