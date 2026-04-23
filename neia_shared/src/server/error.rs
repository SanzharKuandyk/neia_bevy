use std::borrow::Cow;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::EntityId;

/// Flexible entity identifier supporting UUID, String, or no ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerError {
    // === Network & Protocol Errors ===
    RenetError(Option<Cow<'static, str>>),
    InitializationError(Option<Cow<'static, str>>),
    UnexpectedShutdown(Option<Cow<'static, str>>),

    InvalidMessage(Option<Cow<'static, str>>),
    DecodeError(Option<Cow<'static, str>>),
    EncodeError(Option<Cow<'static, str>>),

    // === Resource Not Found ===
    /// Generic not found with flexible ID types (UUID, String, or None)
    NotFound {
        kind: Cow<'static, str>,
        id: EntityId,
        context: Option<Cow<'static, str>>,
    },

    // === State & Validation Errors ===
    /// Invalid game state (no teams, wrong phase, etc.)
    InvalidState {
        reason: Cow<'static, str>,
        expected: Option<Cow<'static, str>>,
        actual: Option<Cow<'static, str>>,
    },

    /// Input validation failed
    ValidationFailed {
        field: Cow<'static, str>,
        reason: Cow<'static, str>,
    },

    // === Permission & Access Errors ===
    /// Player doesn't have permission
    PermissionDenied {
        action: Cow<'static, str>,
        reason: Option<Cow<'static, str>>,
    },

    /// Resource is on cooldown or unavailable
    ResourceUnavailable {
        resource: Cow<'static, str>,
        reason: Cow<'static, str>,
        retry_after: Option<u64>, // ticks/seconds
    },

    /// Registry lookup failed (role, etc.)
    RegistryError {
        registry: Cow<'static, str>,
        id: Cow<'static, str>,
    },

    // === Configuration Errors ===
    ConfigurationError {
        config_type: Option<Cow<'static, str>>,
        reason: Cow<'static, str>,
    },

    // === Deprecated (for backwards compatibility) ===
    #[deprecated(note = "Use more specific error types (InvalidState, ValidationFailed, etc.)")]
    InvalidAction(Option<Cow<'static, str>>),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Network & Protocol Errors
            ServerError::RenetError(Some(msg)) => write!(f, "{msg}"),
            ServerError::RenetError(None) => write!(f, "Renet error occurred."),
            ServerError::InitializationError(Some(msg)) => write!(f, "{msg}"),
            ServerError::InitializationError(None) => write!(f, "Server initialization failed."),
            ServerError::UnexpectedShutdown(Some(msg)) => write!(f, "{msg}"),
            ServerError::UnexpectedShutdown(None) => write!(f, "Unexpected server shutdown."),
            ServerError::InvalidMessage(Some(msg)) => write!(f, "{msg}"),
            ServerError::InvalidMessage(None) => write!(f, "Invalid client message received."),
            ServerError::DecodeError(Some(msg)) => write!(f, "{msg}"),
            ServerError::DecodeError(None) => write!(f, "Failed to decode client message."),
            ServerError::EncodeError(Some(msg)) => write!(f, "{msg}"),
            ServerError::EncodeError(None) => write!(f, "Failed to encode server response."),

            // Resource Not Found
            ServerError::NotFound { kind, id, context } => match (id, context) {
                (EntityId::None, None) => write!(f, "{kind} not found."),
                (EntityId::None, Some(ctx)) => write!(f, "{kind} not found: {ctx}"),
                (id, None) => write!(f, "{kind} with id {id} not found."),
                (id, Some(ctx)) => write!(f, "{kind} with id {id} not found: {ctx}"),
            },

            // State & Validation Errors
            ServerError::InvalidState {
                reason,
                expected,
                actual,
            } => match (expected, actual) {
                (None, None) => write!(f, "Invalid state: {reason}"),
                (Some(exp), None) => write!(f, "Invalid state: {reason} (expected: {exp})"),
                (None, Some(act)) => write!(f, "Invalid state: {reason} (actual: {act})"),
                (Some(exp), Some(act)) => {
                    write!(
                        f,
                        "Invalid state: {reason} (expected: {exp}, actual: {act})"
                    )
                }
            },

            ServerError::ValidationFailed { field, reason } => {
                write!(f, "Validation failed for '{field}': {reason}")
            }

            // Permission & Access Errors
            ServerError::PermissionDenied { action, reason } => match reason {
                Some(r) => write!(f, "Permission denied for '{action}': {r}"),
                None => write!(f, "Permission denied for '{action}'"),
            },

            ServerError::ResourceUnavailable {
                resource,
                reason,
                retry_after,
            } => match retry_after {
                Some(ticks) => write!(
                    f,
                    "Resource '{resource}' unavailable: {reason} (retry after {ticks} ticks)"
                ),
                None => write!(f, "Resource '{resource}' unavailable: {reason}"),
            },

            ServerError::RegistryError { registry, id } => {
                write!(f, "{registry} '{id}' not found in registry")
            }

            // Configuration Errors
            ServerError::ConfigurationError {
                config_type,
                reason,
            } => match config_type {
                Some(t) => write!(f, "Configuration error ({t}): {reason}"),
                None => write!(f, "Configuration error: {reason}"),
            },

            // Deprecated
            #[allow(deprecated)]
            ServerError::InvalidAction(Some(msg)) => write!(f, "{msg}"),
            #[allow(deprecated)]
            ServerError::InvalidAction(None) => write!(f, "Invalid action attempted."),
        }
    }
}

// === Error Construction Macros ===

/// Generic not_found with flexible ID type (UUID, String, or None)
#[macro_export]
macro_rules! not_found {
    ($kind:expr, $id:expr) => {
        $crate::ServerError::NotFound {
            kind: std::borrow::Cow::Borrowed($kind),
            id: $crate::EntityId::from($id),
            context: None,
        }
    };
    ($kind:expr, $id:expr, $context:expr) => {
        $crate::ServerError::NotFound {
            kind: std::borrow::Cow::Borrowed($kind),
            id: $crate::EntityId::from($id),
            context: Some(std::borrow::Cow::Borrowed($context)),
        }
    };
}

/// Specific entity not found macros (UUID-based)
#[macro_export]
macro_rules! player_not_found {
    ($id:expr) => {
        $crate::not_found!("Player", $id)
    };
}

#[macro_export]
macro_rules! team_not_found {
    ($id:expr) => {
        $crate::not_found!("Team", $id)
    };
}

#[macro_export]
macro_rules! item_not_found {
    ($id:expr) => {
        $crate::not_found!("Item", $id)
    };
}

#[macro_export]
macro_rules! container_not_found {
    ($id:expr) => {
        $crate::not_found!("Container", $id)
    };
}

#[macro_export]
macro_rules! object_not_found {
    ($id:expr) => {
        $crate::not_found!("Object", $id)
    };
}

/// Registry entity not found macros (String-based IDs)
#[macro_export]
macro_rules! role_not_found {
    ($id:expr) => {
        $crate::ServerError::RegistryError {
            registry: std::borrow::Cow::Borrowed("role"),
            id: std::borrow::Cow::Owned($id.to_string()),
        }
    };
}

/// State error helpers
#[macro_export]
macro_rules! invalid_state {
    ($reason:expr) => {
        $crate::ServerError::InvalidState {
            reason: std::borrow::Cow::Owned($reason.to_string()),
            expected: None,
            actual: None,
        }
    };
    ($reason:expr, $expected:expr, $actual:expr) => {
        $crate::ServerError::InvalidState {
            reason: std::borrow::Cow::Owned($reason.to_string()),
            expected: Some(std::borrow::Cow::Owned($expected.to_string())),
            actual: Some(std::borrow::Cow::Owned($actual.to_string())),
        }
    };
}

/// Permission denied helper
#[macro_export]
macro_rules! permission_denied {
    ($action:expr) => {
        $crate::ServerError::PermissionDenied {
            action: std::borrow::Cow::Borrowed($action),
            reason: None,
        }
    };
    ($action:expr, $reason:expr) => {
        $crate::ServerError::PermissionDenied {
            action: std::borrow::Cow::Borrowed($action),
            reason: Some(std::borrow::Cow::Borrowed($reason)),
        }
    };
}

/// Validation failed helper
#[macro_export]
macro_rules! validation_failed {
    ($field:expr, $reason:expr) => {
        $crate::ServerError::ValidationFailed {
            field: std::borrow::Cow::Owned($field.to_string()),
            reason: std::borrow::Cow::Owned($reason.to_string()),
        }
    };
}
