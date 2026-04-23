use std::borrow::Cow;
use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Flexible entity identifier supporting UUID, String, or no ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityId {
    Uuid(Uuid),
    String(Cow<'static, str>),
    None,
}

impl From<Uuid> for EntityId {
    fn from(id: Uuid) -> Self {
        EntityId::Uuid(id)
    }
}

impl From<String> for EntityId {
    fn from(id: String) -> Self {
        EntityId::String(id.into())
    }
}

impl From<&str> for EntityId {
    fn from(id: &str) -> Self {
        EntityId::String(Cow::Owned(id.to_string()))
    }
}

impl From<Cow<'static, str>> for EntityId {
    fn from(id: Cow<'static, str>) -> Self {
        EntityId::String(id)
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntityId::Uuid(id) => write!(f, "{}", id),
            EntityId::String(id) => write!(f, "{}", id),
            EntityId::None => write!(f, "<none>"),
        }
    }
}
