use postcard::Error as PostcardError;
use serde::{Deserialize, Serialize};

use crate::ServerError;

use super::{Delta, DeltaData};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerResponse {
    pub tick: u64,
    pub deltas: Vec<DeltaData>,
    pub error: Option<ServerError>,
}

impl ServerResponse {
    pub fn single<D: Delta + 'static>(tick: u64, delta: D) -> Result<Self, PostcardError> {
        let data = delta.to_bytes()?;
        let delta_type = delta.delta_type().to_string();
        let priority = delta.priority();
        Ok(ServerResponse {
            tick,
            deltas: vec![DeltaData {
                delta_type,
                data,
                priority,
            }],
            error: None,
        })
    }

    pub fn error(tick: u64, error: ServerError) -> Self {
        ServerResponse {
            tick,
            deltas: vec![],
            error: Some(error),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PostcardError> {
        postcard::from_bytes(bytes)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, PostcardError> {
        postcard::to_allocvec(self)
    }
}
