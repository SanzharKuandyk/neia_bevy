use async_trait::async_trait;

use crate::ServerError;
use crate::server::context::ServerContext;

use super::MessageTarget;

/// Handler response modes:
/// - `Success`: Dispatcher builds response from accumulated deltas in context and sends to target
/// - `Handled`: Handler managed all response sending internally (no automatic dispatch)
#[derive(Debug, Clone)]
pub enum HandlerResponse {
    /// Dispatcher builds response from ctx.take_deltas() and sends to the target.
    /// Useful for standard message handling where all state changes are accumulated via push_delta().
    Success(MessageTarget),
    /// Handler succeeded and managed all client communication internally.
    /// Use this for custom routing logic or when modders need full control.
    /// **Note for modders**: You are responsible for calling ctx.send_to() or ctx.broadcast()
    /// and ensuring all deltas from your state mutations are properly sent.
    Handled,
}

/// A server-side handler that processes messages of a given type `M`.
#[async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle(&self, ctx: &mut dyn ServerContext) -> Result<HandlerResponse, ServerError>;
}
