use bevy::prelude::*;
use bevy_fluent::Localization;

use super::{ButtonSpec, MenuButtonAction, ScreenUiRoot, render_screen_ui};
use crate::locale::t;
use crate::state::{GameSession, Pause};

pub(crate) fn enter_gameplay(
    mut commands: Commands,
    existing: Query<Entity, With<ScreenUiRoot>>,
    game_session: Res<GameSession>,
    pause: Res<State<Pause>>,
    localization: Option<Res<Localization>>,
) {
    render(
        &mut commands,
        existing.iter().next(),
        &game_session,
        pause.get().0,
        localization.as_deref(),
    );
}

pub(crate) fn refresh_gameplay_ui(
    mut commands: Commands,
    existing: Query<Entity, With<ScreenUiRoot>>,
    game_session: Res<GameSession>,
    pause: Res<State<Pause>>,
    localization: Option<Res<Localization>>,
) {
    render(
        &mut commands,
        existing.iter().next(),
        &game_session,
        pause.get().0,
        localization.as_deref(),
    );
}

fn render(
    commands: &mut Commands,
    existing: Option<Entity>,
    game_session: &GameSession,
    paused: bool,
    localization: Option<&Localization>,
) {
    let mut lines = vec![format!(
        "tick={} started={}",
        game_session.last_tick, game_session.started
    )];
    let buttons = if paused {
        lines.push(t(localization, "pause-help"));
        vec![
            ButtonSpec::new(
                t(localization, "button-continue"),
                MenuButtonAction::ResumeGameplay,
            ),
            ButtonSpec::new(
                t(localization, "button-settings"),
                MenuButtonAction::OpenSettings,
            ),
            ButtonSpec::new(
                t(localization, "button-quit-to-title"),
                MenuButtonAction::QuitToTitle,
            ),
        ]
    } else {
        lines.push(t(localization, "gameplay-pause-help"));
        Vec::new()
    };

    render_screen_ui(
        commands,
        existing,
        t(localization, "gameplay-title"),
        lines,
        buttons,
    );
}

pub fn gameplay_input() {}
