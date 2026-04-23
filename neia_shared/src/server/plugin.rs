use serde::{Deserialize, Serialize};

/// Metadata describing a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
}

impl PluginInfo {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: None,
            author: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }
}

/// A plugin that registers its content into the server registry.
///
/// This is the modding/extensibility API. Implement this trait to add
/// your game's messages, deltas, and any future extension types.
///
/// ```ignore
/// struct MyGamePlugin;
///
/// impl Plugin for MyGamePlugin {
///     fn info(&self) -> PluginInfo {
///         PluginInfo::new("my_game", "0.1.0")
///     }
///
///     fn build(&self, registry: &mut RegistryBuilder) {
///         registry
///             .register_message::<Move>("game.move")
///             .register_delta::<PlayerMoved>("game.player_moved");
///     }
/// }
/// ```
pub trait Plugin: Send + Sync {
    fn info(&self) -> PluginInfo;

    /// Called once at startup. Register messages, deltas, etc.
    fn build(&self, registry: &mut super::registry::RegistryBuilder);
}
