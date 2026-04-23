use neia_shared::client::message::Message;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTeam {
    pub name: String,
}

impl Message for AddTeam {
    fn name(&self) -> &'static str {
        "sys.add_team"
    }
}
