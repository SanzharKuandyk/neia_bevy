use std::any::Any;
use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;
use hashbrown::HashMap;

use crate::ServerError;
use crate::client::message::Message;

use super::DeltaData;
use super::context::ServerContext;
use super::delta::Delta;
use super::message::MessageTarget;
use super::message::handler::{HandlerResponse, MessageHandler};
use super::message::validator::MessageValidator;
use super::plugin::{Plugin, PluginInfo};

/// Result of dispatching a game message through its handler.
pub enum DispatchOutcome {
    /// Handler succeeded; deltas should be routed to `target`.
    Response {
        target: MessageTarget,
        deltas: Vec<DeltaData>,
    },
    /// Handler managed all sending internally.
    Handled,
}

/// Classification after registry lookup.
pub enum DispatchClass {
    /// System message: decoded, ready for server-side handling (downcast via `Any`).
    System(Box<dyn Any + Send>),
    /// Game message: handler that requires a `ServerContext` to execute.
    Game(Arc<dyn ErasedGameHandler>),
}

/// Type-erased game message handler (decode + validate + handle).
#[async_trait]
pub trait ErasedGameHandler: Send + Sync {
    async fn handle_game(
        &self,
        ctx: &mut dyn ServerContext,
        payload: &[u8],
    ) -> Result<HandlerResponse, ServerError>;
}

/// Concrete implementation for any `M` that is Message + Handler + Validator.
struct GameHandler<M>(PhantomData<M>);

#[async_trait]
impl<M> ErasedGameHandler for GameHandler<M>
where
    M: Message + MessageHandler + MessageValidator + Send + Sync + 'static,
{
    async fn handle_game(
        &self,
        ctx: &mut dyn ServerContext,
        payload: &[u8],
    ) -> Result<HandlerResponse, ServerError> {
        let msg: M = M::from_bytes(payload)
            .map_err(|e| ServerError::DecodeError(Some(e.to_string().into())))?;
        msg.validate(ctx).await?;
        msg.handle(ctx).await
    }
}

/// Type-erased system message handler (decode only, no game context).
trait ErasedSystemHandler: Send + Sync {
    fn decode_only(&self, payload: &[u8]) -> Result<Box<dyn Any + Send>, ServerError>;
}

struct SystemHandler<M>(PhantomData<M>);

impl<M> ErasedSystemHandler for SystemHandler<M>
where
    M: Message + Send + Sync + 'static,
{
    fn decode_only(&self, payload: &[u8]) -> Result<Box<dyn Any + Send>, ServerError> {
        let msg: M = M::from_bytes(payload)
            .map_err(|e| ServerError::DecodeError(Some(e.to_string().into())))?;
        Ok(Box::new(msg))
    }
}

/// Type-erased delta deserializer (for client-side reconstruction).
pub trait ErasedDeltaDeserializer: Send + Sync {
    fn deserialize(&self, bytes: &[u8]) -> Result<Box<dyn Any + Send>, ServerError>;
}

struct DeltaDeserializer<D>(PhantomData<D>);

impl<D> ErasedDeltaDeserializer for DeltaDeserializer<D>
where
    D: Delta + Send + Sync + 'static,
{
    fn deserialize(&self, bytes: &[u8]) -> Result<Box<dyn Any + Send>, ServerError> {
        let delta: D = D::from_bytes(bytes)
            .map_err(|e| ServerError::DecodeError(Some(e.to_string().into())))?;
        Ok(Box::new(delta))
    }
}

enum ErasedHandler {
    Game(Arc<dyn ErasedGameHandler>),
    System(Box<dyn ErasedSystemHandler>),
}

/// Builder for constructing an immutable `Registry`.
///
/// Call `register_*` methods (or `add_plugin`) then `.build()`.
pub struct RegistryBuilder {
    messages: HashMap<String, ErasedHandler>,
    deltas: HashMap<String, Box<dyn ErasedDeltaDeserializer>>,
    plugins: Vec<PluginInfo>,
}

impl RegistryBuilder {
    pub fn new() -> Self {
        Self {
            messages: HashMap::new(),
            deltas: HashMap::new(),
            plugins: Vec::new(),
        }
    }

    /// Register a game message (requires Handler + Validator).
    pub fn register_message<M>(&mut self, name: &str) -> &mut Self
    where
        M: Message + MessageHandler + MessageValidator + Send + Sync + 'static,
    {
        let handler: Arc<dyn ErasedGameHandler> = Arc::new(GameHandler::<M>(PhantomData));
        self.messages
            .insert(name.to_string(), ErasedHandler::Game(handler));
        self
    }

    /// Register a system message (decode-only, no handler/validator).
    pub fn register_system_message<M>(&mut self, name: &str) -> &mut Self
    where
        M: Message + Send + Sync + 'static,
    {
        self.messages.insert(
            name.to_string(),
            ErasedHandler::System(Box::new(SystemHandler::<M>(PhantomData))),
        );
        self
    }

    /// Register a delta type for client-side deserialization.
    pub fn register_delta<D>(&mut self, name: &str) -> &mut Self
    where
        D: Delta + Send + Sync + 'static,
    {
        self.deltas.insert(
            name.to_string(),
            Box::new(DeltaDeserializer::<D>(PhantomData)),
        );
        self
    }

    /// Apply a plugin's registrations.
    pub fn add_plugin(&mut self, plugin: &dyn Plugin) -> &mut Self {
        self.plugins.push(plugin.info());
        plugin.build(self);
        self
    }

    /// Consume the builder and produce an immutable `Registry`.
    pub fn build(self) -> Registry {
        Registry {
            messages: self.messages,
            deltas: self.deltas,
            plugins: self.plugins,
        }
    }
}

impl Default for RegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Immutable message/delta registry. No locks needed for reads.
///
/// Created via `RegistryBuilder::build()` at server startup.
///
/// Note: Not `Debug` because it holds trait objects. Use `registry.plugins()` to inspect contents.
pub struct Registry {
    messages: HashMap<String, ErasedHandler>,
    deltas: HashMap<String, Box<dyn ErasedDeltaDeserializer>>,
    plugins: Vec<PluginInfo>,
}

impl Registry {
    /// Classify an incoming framed message without game context.
    ///
    /// - System messages (`sys.*`) are decoded immediately and returned as `Box<dyn Any>`.
    /// - Game messages are returned as a lazy handler; execution happens later with context.
    pub fn classify_message(
        &self,
        name: &str,
        payload: &[u8],
    ) -> Result<DispatchClass, ServerError> {
        let handler = self.messages.get(name).ok_or_else(|| {
            ServerError::InvalidMessage(Some(format!("No handler registered for '{name}'").into()))
        })?;

        match handler {
            ErasedHandler::Game(game) => Ok(DispatchClass::Game(Arc::clone(game))),
            ErasedHandler::System(sys) => Ok(DispatchClass::System(sys.decode_only(payload)?)),
        }
    }

    /// Execute a previously-classified game message with full context.
    pub async fn execute_game_message(
        &self,
        handler: Arc<dyn ErasedGameHandler>,
        ctx: &mut dyn ServerContext,
        payload: &[u8],
    ) -> Result<DispatchOutcome, ServerError> {
        match handler.handle_game(ctx, payload).await {
            Ok(HandlerResponse::Success(target)) => {
                let deltas = ctx.take_deltas();
                Ok(DispatchOutcome::Response { target, deltas })
            }
            Ok(HandlerResponse::Handled) => Ok(DispatchOutcome::Handled),
            Err(err) => {
                ctx.send_error(err).await?;
                Ok(DispatchOutcome::Handled)
            }
        }
    }

    /// Deserialize a delta from bytes using the registered deserializer.
    pub fn deserialize_delta(
        &self,
        name: &str,
        bytes: &[u8],
    ) -> Result<Box<dyn Any + Send>, ServerError> {
        let deser = self
            .deltas
            .get(name)
            .ok_or_else(|| ServerError::RegistryError {
                registry: "delta".into(),
                id: name.to_string().into(),
            })?;
        deser.deserialize(bytes)
    }

    /// List all loaded plugin infos.
    pub fn plugins(&self) -> &[PluginInfo] {
        &self.plugins
    }
}
