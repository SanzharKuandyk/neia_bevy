use async_trait::async_trait;
use uuid::Uuid;

use crate::ServerError;
use crate::game::config::GameConfig;
use crate::game::player::Player;
use crate::game::state::GameState;

use super::DeltaData;
use super::channel::ServerChannel;
use super::message::MessageTarget;
use super::plugin::PluginInfo;
use super::response::ServerResponse;

#[async_trait]
pub trait ServerContext: Send + Sync {
    /// Current tick, constant during a single handler execution.
    fn tick(&self) -> u64;

    /// Network client id of the message sender.
    fn cid(&self) -> u64;

    /// Resolve a player's network client id.
    fn cid_by_player_id(&self, player_id: Uuid) -> Result<u64, ServerError>;

    /// UUID of the player who sent the message.
    fn player_id(&self) -> Uuid;

    /// Plugins currently loaded on the server.
    fn active_plugins(&self) -> &[PluginInfo] {
        &[]
    }

    /// Immutable game config for this session.
    fn config(&self) -> &GameConfig;

    fn game_state(&self) -> &GameState;
    fn game_state_mut(&mut self) -> &mut GameState;

    /// Send a response to the specified target.
    async fn send_to(
        &mut self,
        target: MessageTarget,
        resp: ServerResponse,
        channel: ServerChannel,
    ) -> Result<(), ServerError>;

    /// Send an error response back to the message sender.
    async fn send_error(&mut self, err: ServerError) -> Result<(), ServerError>;

    /// Accumulate a delta for this handler's response.
    fn push_delta(&mut self, delta: DeltaData) -> Result<(), ServerError>;

    /// Take all accumulated deltas (consumes them).
    fn take_deltas(&mut self) -> Vec<DeltaData>;

    // ── Convenience helpers ────────────────────────────────────────

    async fn broadcast(
        &mut self,
        resp: ServerResponse,
        channel: ServerChannel,
    ) -> Result<(), ServerError> {
        self.send_to(MessageTarget::All, resp, channel).await
    }

    async fn send_to_player(
        &mut self,
        resp: ServerResponse,
        channel: ServerChannel,
    ) -> Result<(), ServerError> {
        self.send_to(MessageTarget::Single(self.cid()), resp, channel)
            .await
    }

    fn player(&self) -> Result<&Player, ServerError> {
        self.game_state().get_player(self.player_id())
    }

    fn player_mut(&mut self) -> Result<&mut Player, ServerError> {
        let pid = self.player_id();
        self.game_state_mut().get_player_mut(pid)
    }

    fn has_role(&self, role_id: &str) -> bool {
        self.player()
            .is_ok_and(|p| p.role_id.as_ref().is_some_and(|r| r == role_id))
    }

    fn player_role_id(&self) -> Result<Option<String>, ServerError> {
        Ok(self.player()?.role_id.clone())
    }
}
