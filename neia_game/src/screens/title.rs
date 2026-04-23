use bevy::prelude::*;
use bevy_fluent::Localization;

use super::{ButtonSpec, MenuButtonAction, ScreenUiRoot, render_screen_ui};
use crate::locale::t;
use crate::messages::MenuAction;
use crate::net::{NetworkClientResource, connect_default};
use crate::state::{ConfigResource, LocalProfile, NetworkState, Screen};

pub(crate) fn enter_title(
    mut commands: Commands,
    existing: Query<Entity, With<ScreenUiRoot>>,
    localization: Option<Res<Localization>>,
) {
    render_screen_ui(
        &mut commands,
        existing.iter().next(),
        "Neia Bevy",
        vec![t(localization.as_deref(), "title-help")],
        vec![
            ButtonSpec::new(
                t(localization.as_deref(), "button-connect"),
                MenuButtonAction::Connect,
            ),
            ButtonSpec::new(
                t(localization.as_deref(), "button-settings"),
                MenuButtonAction::OpenSettings,
            ),
            ButtonSpec::new(
                t(localization.as_deref(), "button-quit"),
                MenuButtonAction::QuitApp,
            ),
        ],
    );
}

pub fn title_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    profile: Res<LocalProfile>,
    config: Res<ConfigResource>,
    mut network_client: ResMut<NetworkClientResource>,
    mut network_state: ResMut<NetworkState>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut actions: MessageWriter<MenuAction>,
) {
    if keyboard.just_pressed(KeyCode::KeyC) {
        match connect_default(&mut network_client, &mut network_state, &config, &profile) {
            Ok(()) => {
                next_screen.set(Screen::Lobby);
            }
            Err(error) => {
                eprintln!("Connect failed: {error}");
            }
        }
    }

    if keyboard.just_pressed(KeyCode::KeyS) {
        actions.write(MenuAction::OpenSettings);
    }

    if keyboard.just_pressed(KeyCode::KeyQ) {
        actions.write(MenuAction::QuitApp);
    }
}
