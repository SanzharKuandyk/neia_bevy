use std::time::Instant;

use neia_shared::ServerError;
use neia_shared::server::channel::ServerChannel;
use neia_shared::server::response::ServerResponse;
use renet::RenetClient;
use renet_netcode::NetcodeClientTransport;

use crate::error::ClientStatus;
use crate::events::{ClientEvent, IncomingBatch};

pub(crate) fn poll(
    client: &mut RenetClient,
    transport: &mut NetcodeClientTransport,
    last_poll: &mut Instant,
    last_status: &mut ClientStatus,
) -> (Vec<ClientEvent>, Vec<IncomingBatch>) {
    let mut events = Vec::new();
    let mut batches = Vec::new();
    let now = Instant::now();
    let dt = now.saturating_duration_since(*last_poll);
    *last_poll = now;

    client.update(dt);
    if let Err(err) = transport.update(dt, client) {
        events.push(ClientEvent::Error(ServerError::RenetError(Some(
            err.to_string().into(),
        ))));
    }
    if let Err(err) = transport.send_packets(client) {
        events.push(ClientEvent::Error(ServerError::RenetError(Some(
            err.to_string().into(),
        ))));
    }

    let status = current_status(client, transport);
    if status != *last_status {
        *last_status = status.clone();
        events.push(ClientEvent::StatusChanged(status));
    }

    for channel in [
        ServerChannel::Snapshots,
        ServerChannel::Gameplay,
        ServerChannel::Errors,
        ServerChannel::Chat,
    ] {
        collect_channel_batches(client, channel, &mut events, &mut batches);
    }

    (events, batches)
}

fn collect_channel_batches(
    client: &mut RenetClient,
    channel: ServerChannel,
    events: &mut Vec<ClientEvent>,
    batches: &mut Vec<IncomingBatch>,
) {
    while let Some(bytes) = client.receive_message(channel) {
        match ServerResponse::from_bytes(bytes.as_ref()) {
            Ok(response) => push_response(channel, response, events, batches),
            Err(err) => {
                events.push(ClientEvent::Error(ServerError::DecodeError(Some(
                    err.to_string().into(),
                ))));
            }
        }
    }
}

fn push_response(
    channel: ServerChannel,
    response: ServerResponse,
    events: &mut Vec<ClientEvent>,
    batches: &mut Vec<IncomingBatch>,
) {
    let ServerResponse {
        tick,
        deltas,
        error,
    } = response;

    if let Some(error) = error {
        events.push(ClientEvent::Error(error));
    }

    if deltas.is_empty() {
        return;
    }

    batches.push(IncomingBatch {
        channel,
        tick,
        deltas,
    });
}

fn current_status(client: &RenetClient, transport: &NetcodeClientTransport) -> ClientStatus {
    if let Some(reason) = transport.disconnect_reason() {
        return ClientStatus::ConnectionError(reason.to_string());
    }
    if client.is_disconnected() {
        if let Some(reason) = client.disconnect_reason() {
            return ClientStatus::ConnectionError(reason.to_string());
        }
        return ClientStatus::Disconnected;
    }
    if client.is_connected() {
        return ClientStatus::Connected;
    }
    ClientStatus::Connecting
}
