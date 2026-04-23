use std::sync::Arc;

use neia_shared::server::error::ServerError;
use neia_shared::{UserData, validation_failed};
use renet::{RenetServer, ServerEvent};
use renet_netcode::NetcodeServerTransport;

use crate::AppState;
use crate::server::sender::send_err_message;

mod connect;
mod disconnect;
mod helpers;
mod spawn;

pub use connect::handle_player_connect;
pub use disconnect::handle_player_disconnect;

pub mod reconnect;

pub async fn handle_events(
    server: &mut RenetServer,
    transport: &mut NetcodeServerTransport,
    app_state: Arc<AppState>,
    tick: u64,
) {
    while let Some(event) = server.get_event() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                tracing::debug!("Connected user client id: {}", client_id);

                if let Some(data) = transport.user_data(client_id) {
                    match postcard::from_bytes::<UserData>(&data) {
                        Ok(user_data) => {
                            handle_player_connect(server, &app_state, client_id, user_data, tick)
                                .await;
                        }
                        Err(e) => {
                            tracing::error!(
                                "Failed to decode user data for client {}: {}",
                                client_id,
                                e
                            );
                            let error = ServerError::DecodeError(Some("Invalid user data".into()));
                            let _ = send_err_message(server, client_id, error, tick).await;
                            server.disconnect(client_id);
                        }
                    }
                } else {
                    tracing::warn!("No user data found for client {}", client_id);
                    let error = validation_failed!("user_data", "No user data provided");
                    let _ = send_err_message(server, client_id, error, tick).await;
                    server.disconnect(client_id);
                }
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                tracing::debug!("Disconnected user client id: {client_id}, reason: {reason}");
                handle_player_disconnect(server, &app_state, client_id, tick).await;
            }
        }
    }
}
