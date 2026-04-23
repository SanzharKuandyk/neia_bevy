use std::sync::Arc;

use neia_defaults::deltas::game_started::GameStarted;
use neia_shared::server::DeltaData;
use neia_shared::server::channel::ServerChannel;
use neia_shared::server::message::MessageTarget;
use neia_shared::server::response::ServerResponse;
use renet::{ClientId, RenetServer};

use crate::AppState;
use crate::server::sender::{send_err_message, send_message};

pub async fn handle_game_start(
    server: &mut RenetServer,
    app_state: &Arc<AppState>,
    client_id: ClientId,
    tick: u64,
    _delta_secs: f32,
) {
    // Grab a snapshot of the current game config for this session
    let game_config = app_state.get_game_config().await;

    let start_result = {
        let mut session_guard = app_state.server_state.session.write().await;
        session_guard.start_game(game_config, tick)
    };

    match start_result {
        Ok(mut round_deltas) => {
            tracing::info!("Game started successfully");

            let mut all_deltas = Vec::new();

            // GameStarted delta first
            match DeltaData::from_delta(&GameStarted) {
                Ok(delta_data) => all_deltas.push(delta_data),
                Err(e) => tracing::error!("Failed to serialize GameStarted delta: {}", e),
            }

            all_deltas.append(&mut round_deltas);

            let response = ServerResponse {
                tick,
                deltas: all_deltas,
                error: None,
            };
            if let Err(err) = send_message(
                server,
                MessageTarget::All,
                response,
                ServerChannel::Gameplay,
                None,
            )
            .await
            {
                let _ = send_err_message(server, client_id, err, tick).await;
            }
        }
        Err(e) => {
            tracing::warn!("Failed to start game: {}", e);
            let _ = send_err_message(server, client_id, e, tick).await;
        }
    }
}
