use neia_shared::client::message::Message;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartGame;

impl Message for StartGame {
    fn name(&self) -> &'static str {
        "sys.start_game"
    }
}
