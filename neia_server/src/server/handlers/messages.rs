use std::sync::Arc;

use neia_defaults::deltas::chat_message::ChatMessage;
use neia_defaults::messages::chat::Chat;
use neia_shared::client::message::Message;
use neia_shared::client::message::split_message_header;
use neia_shared::server::DeltaData;
use neia_shared::server::channel::ServerChannel;
use neia_shared::server::context::ServerContext;
use neia_shared::server::message::MessageTarget;
use neia_shared::server::registry::{DispatchClass, DispatchOutcome};
use neia_shared::server::response::ServerResponse;
use neia_shared::{ServerError, invalid_state};
use renet::{ClientId, RenetServer};

use crate::AppState;
use crate::server::sender::send_err_message;

use super::context::HandlerContext;

mod system_message_handler;

use system_message_handler::handle_sys_message;

/// Process all client messages for this tick.
/// Returns accumulated deltas for batching with session deltas.
pub async fn handle_messages(
    server: &mut RenetServer,
    app_state: Arc<AppState>,
    tick: u64,
    max_messages_per_tick: usize,
) -> Vec<DeltaData> {
    let mut all_deltas = Vec::new();

    let client_ids = server.clients_id();

    for client_id in client_ids {
        let player_id = match get_player_id_for_client(&app_state, client_id).await {
            Some(pid) => pid,
            None => continue,
        };

        let mut deltas = process_reliable_messages(
            server,
            &app_state,
            client_id,
            player_id,
            tick,
            max_messages_per_tick,
        )
        .await;
        all_deltas.append(&mut deltas);

        let mut chat_deltas = process_unreliable_messages(server, client_id, player_id, tick).await;
        all_deltas.append(&mut chat_deltas);
    }

    all_deltas
}

async fn get_player_id_for_client(
    app_state: &Arc<AppState>,
    client_id: ClientId,
) -> Option<uuid::Uuid> {
    let client_map = app_state.server_state.client_map.lock().await;
    match client_map.get(&client_id).copied() {
        Some(pid) => Some(pid),
        None => {
            tracing::warn!("Message from unknown client {:?}", client_id);
            None
        }
    }
}

async fn process_reliable_messages(
    server: &mut RenetServer,
    app_state: &Arc<AppState>,
    client_id: ClientId,
    player_id: uuid::Uuid,
    tick: u64,
    max_messages_per_tick: usize,
) -> Vec<DeltaData> {
    let mut deltas = Vec::new();
    let mut msg_count = 0;

    loop {
        let Some(bytes) = server.receive_message(client_id, ServerChannel::Gameplay) else {
            break;
        };

        if msg_count >= max_messages_per_tick {
            tracing::warn!(
                "Client {} hit max messages per tick limit ({})",
                client_id,
                max_messages_per_tick
            );
            break;
        }

        if let Some((name, payload)) = split_message_header(&bytes) {
            match app_state.registry.classify_message(name, payload) {
                Ok(DispatchClass::System(msg_any)) => {
                    let delta_secs = app_state.config.server_config().delta_time_ms as f32 / 1000.0;
                    handle_sys_message(
                        msg_any, server, app_state, client_id, player_id, tick, delta_secs,
                    )
                    .await;
                }
                Ok(DispatchClass::Game(game_handler)) => {
                    let client_map = app_state.server_state.client_map.lock().await;
                    let mut session_guard = app_state.server_state.session.write().await;
                    let plugins = app_state.registry.plugins();

                    let session = match session_guard.as_game_mut() {
                        Ok(game) => game,
                        Err(_) => {
                            tracing::warn!(
                                "Received game message '{}' but session not in InGame state",
                                name
                            );
                            let _ = send_err_message(
                                server,
                                client_id,
                                invalid_state!(format!(
                                    "Cannot handle game message '{}' outside InGame",
                                    name
                                )),
                                tick,
                            )
                            .await;
                            msg_count += 1;
                            continue;
                        }
                    };

                    let mut ctx = match HandlerContext::new(
                        client_id,
                        player_id,
                        server,
                        session,
                        &client_map,
                        tick,
                        plugins,
                    ) {
                        Ok(ctx) => ctx,
                        Err(err) => {
                            tracing::error!(
                                "Failed to create handler context for client {}: {err:?}",
                                client_id
                            );
                            msg_count += 1;
                            continue;
                        }
                    };

                    match app_state
                        .registry
                        .execute_game_message(game_handler, &mut ctx, payload)
                        .await
                    {
                        Ok(DispatchOutcome::Response {
                            target,
                            deltas: msg_deltas,
                        }) => {
                            if msg_deltas.is_empty() {
                                msg_count += 1;
                                continue;
                            }

                            if matches!(target, MessageTarget::All) {
                                deltas.extend(msg_deltas);
                            } else if let Err(err) = ctx
                                .send_to(
                                    target,
                                    ServerResponse {
                                        tick,
                                        deltas: msg_deltas,
                                        error: None,
                                    },
                                    ServerChannel::Gameplay,
                                )
                                .await
                            {
                                tracing::error!(
                                    "Failed to send targeted response for `{name}`: {err:?}"
                                );
                            }
                        }
                        Ok(DispatchOutcome::Handled) => {}
                        Err(err) => {
                            tracing::error!("Error handling message `{name}`: {err:?}");
                        }
                    }
                }
                Err(err) => {
                    tracing::error!("Error classifying message `{name}`: {err:?}");
                    let _ = send_err_message(server, client_id, err, tick).await;
                }
            }

            msg_count += 1;
        } else {
            tracing::warn!(
                "Failed to parse framed message header from client {}",
                client_id
            );
            let _ = send_err_message(
                server,
                client_id,
                ServerError::InvalidMessage(Some("Malformed framed message".into())),
                tick,
            )
            .await;
        }
    }

    deltas
}

async fn process_unreliable_messages(
    server: &mut RenetServer,
    client_id: ClientId,
    player_id: uuid::Uuid,
    _tick: u64,
) -> Vec<DeltaData> {
    let mut deltas = Vec::new();

    while let Some(bytes) = server.receive_message(client_id, ServerChannel::Chat) {
        if let Ok(chat_msg) = Chat::from_bytes(&bytes) {
            let delta = ChatMessage {
                player_id,
                message: chat_msg.text,
            };

            if let Ok(delta_data) = DeltaData::from_delta(&delta) {
                deltas.push(delta_data);
            } else {
                tracing::error!("Failed to serialize chat delta");
            }
        } else {
            tracing::warn!(
                "Failed to decode Chat message from client {} on unreliable channel",
                client_id
            );
        }
    }

    deltas
}
