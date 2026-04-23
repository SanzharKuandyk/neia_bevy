use neia_shared::client::message::Message;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ready {
    pub ready: bool,
}

impl Message for Ready {
    fn name(&self) -> &'static str {
        "sys.ready"
    }
}
