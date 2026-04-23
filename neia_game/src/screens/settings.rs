use bevy::prelude::*;
use bevy_fluent::Localization;

use super::{ButtonSpec, MenuButtonAction, ScreenUiRoot, render_screen_ui};
use crate::locale::t;
use crate::messages::MenuAction;
use crate::state::{ConfigResource, Pause};

pub(crate) fn enter_settings(
    mut commands: Commands,
    existing: Query<Entity, With<ScreenUiRoot>>,
    config: Res<ConfigResource>,
    pause: Res<State<Pause>>,
    localization: Option<Res<Localization>>,
) {
    let mut lines = vec![
        format!(
            "{}: 100%",
            t(localization.as_deref(), "settings-master-volume")
        ),
        format!(
            "{}: {}",
            t(localization.as_deref(), "settings-default-server"),
            config.app.client.default_server_addr
        ),
    ];

    if pause.get().0 {
        lines.push(t(localization.as_deref(), "settings-back-gameplay"));
    } else {
        lines.push(t(localization.as_deref(), "nav-back"));
    }

    render_screen_ui(
        &mut commands,
        existing.iter().next(),
        t(localization.as_deref(), "settings-title"),
        lines,
        vec![ButtonSpec::new(
            t(localization.as_deref(), "button-back"),
            MenuButtonAction::BackFromSettings,
        )],
    );
}

pub fn settings_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    pause: Res<State<Pause>>,
    mut actions: MessageWriter<MenuAction>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    if pause.get().0 {
        actions.write(MenuAction::ResumeGameplay);
    } else {
        actions.write(MenuAction::BackToTitle);
    }
}
