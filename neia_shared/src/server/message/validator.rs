use async_trait::async_trait;

use crate::ServerError;
use crate::server::context::ServerContext;

#[async_trait]
pub trait MessageValidator {
    async fn validate(&self, _ctx: &dyn ServerContext) -> Result<(), ServerError> {
        Ok(())
    }
}
