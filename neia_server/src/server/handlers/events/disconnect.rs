use neia_defaults::deltas::player_left_game::PlayerLeftGame;
use neia_defaults::deltas::player_left_lobby::PlayerLeftLobby;
use neia_shared::server::channel::ServerChannel;
use neia_shared::server::message::MessageTarget;
use neia_shared::server::response::ServerResponse;
use renet::RenetServer;

use crate::AppState;
use crate::server::sender::send_message;
use crate::server::session::SessionState;

use super::helpers::push_delta;

pub async fn handle_player_disconnect(
    server: &mut RenetServer,
    app_state: &AppState,
    client_id: u64,
    tick: u64,
) {
    let player_id_opt = {
        let mut client_map = app_state.server_state.client_map.lock().await;
        client_map.remove(&client_id)
    };

    if let Some(player_id) = player_id_opt {
        let mut session_guard = app_state.server_state.session.write().await;

        match &mut *session_guard {
            SessionState::Lobby(lobby) => {
                if let Some(lobby_player) = lobby.remove_player(player_id) {
                    tracing::info!("Player {} left lobby", lobby_player.name);

                    let mut deltas = Vec::new();
                    push_delta(&mut deltas, PlayerLeftLobby { player_id });

                    drop(session_guard);

                    let response = ServerResponse {
                        tick,
                        deltas,
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
            SessionState::InGame(game) => {
                let _ = game.state.remove_player(player_id);
                tracing::info!("Player {} left game", player_id);

                let mut deltas = Vec::new();
                push_delta(&mut deltas, PlayerLeftGame { player_id });

                drop(session_guard);

                let response = ServerResponse {
                    tick,
                    deltas,
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
            _ => {}
        }
    }
}
