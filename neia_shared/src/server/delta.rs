use postcard::Error as PostcardError;
use serde::{Deserialize, Serialize};

/// Wrapper for serializable delta data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaData {
    pub delta_type: String,
    pub data: Vec<u8>,
    #[serde(skip)]
    pub priority: DeltaPriority,
}

impl DeltaData {
    pub fn from_delta<D: Delta>(delta: &D) -> Result<Self, PostcardError> {
        Ok(DeltaData {
            delta_type: delta.delta_type().to_string(),
            data: delta.to_bytes()?,
            priority: delta.priority(),
        })
    }
}

/// Priority for delta ordering within a single tick
/// Lower values are sent first
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DeltaPriority {
    /// Critical game state changes (round start, phase transitions, game end)
    Critical = 0,
    /// High-priority player state (spawns, deaths, position updates)
    High = 100,
    /// Normal game actions (item interactions, cooking, serving)
    #[default]
    Normal = 200,
    /// Low-priority feedback (effects applied, progress updates)
    Low = 300,
}

pub trait Delta: Send + Sync + 'static + Serialize + for<'de> Deserialize<'de> {
    fn delta_type(&self) -> &'static str;

    /// Priority for ordering deltas within a tick
    /// Override this to control when your delta is sent relative to others
    fn priority(&self) -> DeltaPriority {
        DeltaPriority::Normal
    }

    fn to_bytes(&self) -> Result<Vec<u8>, PostcardError> {
        postcard::to_allocvec(self)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, PostcardError>
    where
        Self: Sized,
    {
        postcard::from_bytes(bytes)
    }
}
