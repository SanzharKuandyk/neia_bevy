use bevy::prelude::*;

use crate::state::{Pause, Screen};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MenuAction>()
            .add_systems(Update, apply_menu_actions);
    }
}

#[derive(Message, Debug, Clone, Copy)]
pub enum MenuAction {
    OpenSettings,
    BackToTitle,
    ResumeGameplay,
    QuitApp,
}

fn apply_menu_actions(
    mut actions: MessageReader<MenuAction>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_pause: ResMut<NextState<Pause>>,
    mut exit: MessageWriter<AppExit>,
) {
    for action in actions.read() {
        match action {
            MenuAction::OpenSettings => {
                next_screen.set(Screen::Settings);
                next_pause.set(Pause(false));
            }
            MenuAction::BackToTitle => {
                next_screen.set(Screen::Title);
                next_pause.set(Pause(false));
            }
            MenuAction::ResumeGameplay => {
                next_pause.set(Pause(false));
            }
            MenuAction::QuitApp => {
                exit.write(AppExit::Success);
            }
        }
    }
}
