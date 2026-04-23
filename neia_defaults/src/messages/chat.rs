use neia_shared::client::message::Message;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    pub text: String,
}

impl Message for Chat {
    fn name(&self) -> &'static str {
        "sys.chat"
    }
}
