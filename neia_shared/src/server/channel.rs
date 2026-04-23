use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServerChannel {
    Gameplay,
    Snapshots,
    Chat,
    Errors,
}

impl From<ServerChannel> for u8 {
    fn from(channel: ServerChannel) -> u8 {
        match channel {
            ServerChannel::Gameplay => 0,
            ServerChannel::Snapshots => 1,
            ServerChannel::Chat => 2,
            ServerChannel::Errors => 3,
        }
    }
}
