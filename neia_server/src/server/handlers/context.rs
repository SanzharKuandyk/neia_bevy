use async_trait::async_trait;
use hashbrown::HashMap;
use neia_shared::game::config::GameConfig;
use neia_shared::game::state::GameState;
use neia_shared::server::DeltaData;
use neia_shared::server::channel::ServerChannel;
use neia_shared::server::context::ServerContext;
use neia_shared::server::message::MessageTarget;
use neia_shared::server::plugin::PluginInfo;
use neia_shared::server::response::ServerResponse;
use neia_shared::{ServerError, not_found};
use renet::{ClientId, RenetServer};
use uuid::Uuid;

use crate::server::sender::{send_err_message, send_message};
use crate::server::session::GameSession;

pub struct HandlerContext<'a> {
    gcg: GameConfig,
    client_id: u64,
    player_id: Uuid,
    server: &'a mut RenetServer,
    game_state: &'a mut GameState,
    client_map: &'a HashMap<ClientId, Uuid>,
    tick: u64,
    deltas: Vec<DeltaData>,
    plugins: &'a [PluginInfo],
}

impl<'a> HandlerContext<'a> {
    pub fn new(
        client_id: u64,
        player_id: Uuid,
        server: &'a mut RenetServer,
        session: &'a mut GameSession,
        client_map: &'a HashMap<ClientId, Uuid>,
        tick: u64,
        plugins: &'a [PluginInfo],
    ) -> Result<Self, ServerError> {
        Ok(Self {
            client_id,
            player_id,
            server,
            gcg: session.gcg.clone(),
            game_state: &mut session.state,
            client_map,
            tick,
            deltas: Vec::new(),
            plugins,
        })
    }
}

#[async_trait]
impl<'a> ServerContext for HandlerContext<'a> {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn cid(&self) -> u64 {
        self.client_id
    }

    fn cid_by_player_id(&self, player_id: Uuid) -> Result<u64, ServerError> {
        for (cid, pid) in self.client_map.iter() {
            if *pid == player_id {
                return Ok(*cid);
            }
        }
        Err(not_found!("ClientId by player id", player_id))
    }

    fn player_id(&self) -> Uuid {
        self.player_id
    }

    fn active_plugins(&self) -> &[PluginInfo] {
        self.plugins
    }

    fn config(&self) -> &GameConfig {
        &self.gcg
    }

    fn game_state(&self) -> &GameState {
        self.game_state
    }

    fn game_state_mut(&mut self) -> &mut GameState {
        self.game_state
    }

    async fn send_to(
        &mut self,
        target: MessageTarget,
        resp: ServerResponse,
        channel: ServerChannel,
    ) -> Result<(), ServerError> {
        send_message(self.server, target, resp, channel, None).await
    }

    async fn send_error(&mut self, err: ServerError) -> Result<(), ServerError> {
        send_err_message(self.server, self.client_id, err, self.tick).await
    }

    fn push_delta(&mut self, delta: DeltaData) -> Result<(), ServerError> {
        self.deltas.push(delta);
        Ok(())
    }

    fn take_deltas(&mut self) -> Vec<DeltaData> {
        std::mem::take(&mut self.deltas)
    }
}
