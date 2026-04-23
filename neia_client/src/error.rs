use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ClientStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    ConnectionError(String),
}

#[derive(Debug)]
pub enum ClientError {
    InvalidUserData(String),
    Protocol(String),
    Network(String),
    Decode(String),
}

impl Display for ClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUserData(message)
            | Self::Protocol(message)
            | Self::Network(message)
            | Self::Decode(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for ClientError {}
