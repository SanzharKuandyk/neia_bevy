use bevy::prelude::*;
use bevy_fluent::Localization;

use super::{ButtonSpec, MenuButtonAction, ScreenUiRoot, render_screen_ui};
use crate::client::ClientIntent;
use crate::locale::t;
use crate::messages::MenuAction;
use crate::net::{NetworkClientResource, disconnect};
use crate::state::{LobbyReplica, NetworkState, Screen};

pub(crate) fn enter_lobby(
    mut commands: Commands,
    existing: Query<Entity, With<ScreenUiRoot>>,
    lobby: Res<LobbyReplica>,
    localization: Option<Res<Localization>>,
) {
    let mut lines = vec![t(localization.as_deref(), "lobby-help")];
    if lobby.players.is_empty() {
        lines.push(t(localization.as_deref(), "lobby-empty"));
    } else {
        for player in lobby.players.values() {
            lines.push(format!(
                "{} {}",
                player.name,
                if player.is_ready {
                    t(localization.as_deref(), "lobby-player-ready")
                } else {
                    t(localization.as_deref(), "lobby-player-not-ready")
                }
            ));
        }
    }

    render_screen_ui(
        &mut commands,
        existing.iter().next(),
        t(localization.as_deref(), "lobby-title"),
        lines,
        vec![
            ButtonSpec::new(
                t(localization.as_deref(), "button-ready"),
                MenuButtonAction::ToggleReady,
            ),
            ButtonSpec::new(
                t(localization.as_deref(), "button-start-game"),
                MenuButtonAction::StartGame,
            ),
            ButtonSpec::new(
                t(localization.as_deref(), "button-settings"),
                MenuButtonAction::OpenSettings,
            ),
            ButtonSpec::new(
                t(localization.as_deref(), "button-disconnect"),
                MenuButtonAction::DisconnectToTitle,
            ),
        ],
    );
}

pub fn lobby_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    lobby: Res<LobbyReplica>,
    mut intents: MessageWriter<ClientIntent>,
    mut actions: MessageWriter<MenuAction>,
    mut network_client: ResMut<NetworkClientResource>,
    mut network_state: ResMut<NetworkState>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        let any_unready = lobby.players.values().any(|player| !player.is_ready);
        intents.write(ClientIntent::SetReady(any_unready));
    }

    if keyboard.just_pressed(KeyCode::KeyS) {
        intents.write(ClientIntent::StartGame);
    }

    if keyboard.just_pressed(KeyCode::KeyT) {
        actions.write(MenuAction::OpenSettings);
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        disconnect(&mut network_client, &mut network_state);
        next_screen.set(Screen::Title);
    }
}
