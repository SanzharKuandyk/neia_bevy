use neia_defaults::deltas::player_ready_changed::PlayerReadyChanged;
use neia_shared::server::DeltaData;
use neia_shared::server::channel::ServerChannel;
use neia_shared::server::message::MessageTarget;
use neia_shared::server::response::ServerResponse;
use renet::RenetServer;
use uuid::Uuid;

use crate::server::sender::send_message;
use crate::server::session::SessionState;
use crate::server::state::ServerState;

pub async fn handle_ready_msg(
    server: &mut RenetServer,
    server_state: &ServerState,
    player_id: Uuid,
    ready: bool,
    tick: u64,
) {
    {
        let mut session_guard = server_state.session.write().await;
        if let SessionState::Lobby(lobby) = &mut *session_guard
            && let Err(e) = lobby.set_player_ready(player_id, ready)
        {
            tracing::warn!("Failed to set lobby ready state for {player_id}: {e}");
        }
    }

    let delta = PlayerReadyChanged {
        player_id,
        is_ready: ready,
    };

    if let Ok(delta_data) = DeltaData::from_delta(&delta) {
        let response = ServerResponse {
            tick,
            deltas: vec![delta_data],
            error: None,
        };

        let _ = send_message(
            server,
            MessageTarget::All,
            response,
            ServerChannel::Gameplay,
            None,
        )
        .await;
    }
}
