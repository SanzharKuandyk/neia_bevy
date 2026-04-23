use neia_shared::ServerError;
use neia_shared::server::channel::ServerChannel;
use neia_shared::server::delta::DeltaData;

use crate::error::ClientStatus;

#[derive(Debug)]
pub enum ClientEvent {
    StatusChanged(ClientStatus),
    Error(ServerError),
}

#[derive(Debug)]
pub struct IncomingBatch {
    pub channel: ServerChannel,
    pub tick: u64,
    pub deltas: Vec<DeltaData>,
}
