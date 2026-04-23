use neia_shared::server::plugin::{Plugin, PluginInfo};
use neia_shared::server::registry::RegistryBuilder;

pub mod deltas;
pub mod messages;

use deltas::{
    chat_message::ChatMessage, game_started::GameStarted, player_joined_lobby::PlayerJoinedLobby,
    player_joined_team::PlayerJoinedTeam, player_left_game::PlayerLeftGame,
    player_left_lobby::PlayerLeftLobby, player_ready_changed::PlayerReadyChanged,
    team_added::TeamAdded,
};
use messages::{add_team::AddTeam, chat::Chat, ready::Ready, start_game::StartGame};

/// Built-in plugin that registers the core lobby system messages and deltas.
///
/// Always add this plugin when building a `Registry`:
/// ```ignore
/// let registry = RegistryBuilder::new()
///     .add_plugin(&CorePlugin)
///     .add_plugin(&MyGamePlugin)
///     .build();
/// ```
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo::new("core", env!("CARGO_PKG_VERSION"))
            .with_description("Built-in lobby system messages and deltas")
    }

    fn build(&self, registry: &mut RegistryBuilder) {
        // System messages (decode-only, handled by server directly)
        registry
            .register_system_message::<AddTeam>("sys.add_team")
            .register_system_message::<Ready>("sys.ready")
            .register_system_message::<StartGame>("sys.start_game")
            .register_system_message::<Chat>("sys.chat");

        // System deltas
        registry
            .register_delta::<PlayerJoinedLobby>("sys.player_joined_lobby")
            .register_delta::<PlayerLeftLobby>("sys.player_left_lobby")
            .register_delta::<PlayerJoinedTeam>("sys.player_joined_team")
            .register_delta::<PlayerLeftGame>("sys.player_left_game")
            .register_delta::<PlayerReadyChanged>("sys.player_ready_changed")
            .register_delta::<TeamAdded>("sys.team_added")
            .register_delta::<GameStarted>("sys.game_started")
            .register_delta::<ChatMessage>("sys.chat_message");
    }
}
