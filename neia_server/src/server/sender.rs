use neia_shared::server::DeltaData;
use neia_shared::server::channel::ServerChannel;
use neia_shared::server::error::ServerError;
use neia_shared::server::message::MessageTarget;
use neia_shared::server::response::ServerResponse;
use renet::{ClientId, RenetServer};

pub async fn send_message(
    server: &mut RenetServer,
    target: MessageTarget,
    message: ServerResponse,
    channel: ServerChannel,
    test_cid: Option<u64>,
) -> Result<(), ServerError> {
    tracing::debug!("Message: {:?}", message);
    let serialized_message = message.to_bytes().map_err(|e| {
        tracing::error!("Error serializing the message: {}", e);
        ServerError::EncodeError(Some("Error serializing the message".into()))
    })?;

    match target {
        MessageTarget::Single(client_id) => {
            if let Some(cid) = test_cid {
                if cid == client_id
                    && server.can_send_message(client_id, channel, serialized_message.len())
                {
                    server.send_message(client_id, channel, serialized_message);
                }
            } else if server.can_send_message(client_id, channel, serialized_message.len()) {
                server.send_message(client_id, channel, serialized_message);
            }
        }
        MessageTarget::AllExceptOne(except_id) => {
            server.broadcast_message_except(except_id, channel, serialized_message);
        }
        MessageTarget::All => {
            server.broadcast_message(channel, serialized_message);
        }
        MessageTarget::Team(_team_id) => {
            // Team-targeted broadcast requires resolving team members to client_ids.
            // For now, falls back to broadcast. Implement with a client_map lookup
            // when your game needs team-specific messaging.
            server.broadcast_message(channel, serialized_message);
        }
    }

    Ok(())
}

pub async fn send_err_message(
    server: &mut RenetServer,
    client_id: ClientId,
    error: ServerError,
    tick: u64,
) -> Result<(), ServerError> {
    let error_message = ServerResponse::error(tick, error);

    let serialized = error_message
        .to_bytes()
        .map_err(|e| ServerError::EncodeError(Some(e.to_string().into())))?;

    if server.can_send_message(client_id, ServerChannel::Errors, serialized.len()) {
        server.send_message(client_id, ServerChannel::Errors, serialized);
    } else {
        // Check if client is connected
        if !server.is_connected(client_id) {
            tracing::debug!(
                "Cannot send error to client {}: client disconnected",
                client_id
            );
            return Err(ServerError::RenetError(Some("Client disconnected".into())));
        }

        // Message too large for send buffer
        tracing::warn!(
            "Cannot send error to client {}: message too large ({} bytes)",
            client_id,
            serialized.len()
        );
        return Err(ServerError::RenetError(Some(
            "Error message too large".into(),
        )));
    }

    Ok(())
}

/// Broadcast deltas to all clients on reliable ordered channel
/// Returns Ok(true) if deltas were sent, Ok(false) if no deltas, Err if serialization failed
pub fn broadcast_deltas(
    server: &mut RenetServer,
    tick: u64,
    deltas: Vec<DeltaData>,
    channel: ServerChannel,
) -> Result<bool, ServerError> {
    if deltas.is_empty() {
        return Ok(false);
    }

    let response = ServerResponse {
        tick,
        deltas,
        error: None,
    };

    let serialized = response
        .to_bytes()
        .map_err(|e| ServerError::EncodeError(Some(e.to_string().into())))?;

    server.broadcast_message(channel, serialized);
    Ok(true)
}
