use neia_shared::server::DeltaData;
use neia_shared::server::channel::ServerChannel;
use neia_shared::server::delta::Delta;
use neia_shared::server::error::ServerError;
use renet::RenetServer;

use crate::server::sender;

/// Add delta to delta list (reduces boilerplate).
pub fn push_delta<D: Delta>(deltas: &mut Vec<DeltaData>, delta: D) {
    if let Ok(delta_data) = DeltaData::from_delta(&delta) {
        deltas.push(delta_data);
    }
}

/// Broadcast a list of deltas to all clients on the Gameplay channel.
pub async fn broadcast_deltas(
    server: &mut RenetServer,
    tick: u64,
    deltas: Vec<DeltaData>,
) -> Result<(), ServerError> {
    if deltas.is_empty() {
        return Ok(());
    }
    sender::broadcast_deltas(server, tick, deltas, ServerChannel::Gameplay)?;
    Ok(())
}
