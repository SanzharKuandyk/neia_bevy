pub mod gameplay;
pub mod lobby;
pub mod settings;
pub mod title;

use bevy::prelude::*;

use crate::client::ClientIntent;
use crate::net::{NetworkClientResource, connect_default, disconnect};
use crate::state::{ConfigResource, LobbyReplica, LocalProfile, NetworkState, Pause, Screen};

pub struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SettingsReturn>()
            .add_systems(Startup, setup_camera)
            .add_systems(OnEnter(Screen::Title), title::enter_title)
            .add_systems(OnEnter(Screen::Settings), settings::enter_settings)
            .add_systems(OnEnter(Screen::Lobby), lobby::enter_lobby)
            .add_systems(OnEnter(Screen::Gameplay), gameplay::enter_gameplay)
            .add_systems(Update, title::title_input.run_if(in_state(Screen::Title)))
            .add_systems(
                Update,
                settings::settings_input.run_if(in_state(Screen::Settings)),
            )
            .add_systems(Update, lobby::lobby_input.run_if(in_state(Screen::Lobby)))
            .add_systems(
                Update,
                gameplay::gameplay_input.run_if(in_state(Screen::Gameplay)),
            )
            .add_systems(
                Update,
                gameplay::refresh_gameplay_ui
                    .run_if(in_state(Screen::Gameplay).and(resource_changed::<State<Pause>>)),
            )
            .add_systems(
                Update,
                pause_prompt.run_if(in_state(Screen::Gameplay).and(in_state(Pause(false)))),
            )
            .add_systems(Update, update_button_visuals)
            .add_systems(Update, handle_menu_buttons);
    }
}

#[derive(Component)]
pub(crate) struct ScreenUiRoot;

#[derive(Component, Clone, Copy)]
pub(crate) struct MenuButton(pub MenuButtonAction);

#[derive(Clone, Copy)]
pub(crate) enum MenuButtonAction {
    Connect,
    OpenSettings,
    BackFromSettings,
    ToggleReady,
    StartGame,
    DisconnectToTitle,
    ResumeGameplay,
    QuitToTitle,
    QuitApp,
}

#[derive(Resource, Clone, Copy)]
pub(crate) struct SettingsReturn {
    pub screen: Screen,
    pub pause: Pause,
}

impl Default for SettingsReturn {
    fn default() -> Self {
        Self {
            screen: Screen::Title,
            pause: Pause(false),
        }
    }
}

pub(crate) struct ButtonSpec {
    pub label: String,
    pub action: MenuButtonAction,
}

impl ButtonSpec {
    pub(crate) fn new(label: impl Into<String>, action: MenuButtonAction) -> Self {
        Self {
            label: label.into(),
            action,
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub(crate) fn render_screen_ui(
    commands: &mut Commands,
    root: Option<Entity>,
    title: impl Into<String>,
    lines: Vec<String>,
    buttons: Vec<ButtonSpec>,
) {
    if let Some(root) = root {
        commands.entity(root).despawn();
    }

    let title = title.into();
    commands
        .spawn((
            ScreenUiRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(24.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.05, 0.06, 0.08)),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(720.0),
                        max_width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(12.0),
                        padding: UiRect::all(Val::Px(24.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.12, 0.14, 0.18, 0.96)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(title),
                        TextFont {
                            font_size: 34.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    for line in lines {
                        parent.spawn((
                            Text::new(line),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.86, 0.89, 0.94)),
                        ));
                    }

                    for button in buttons {
                        parent
                            .spawn((
                                Button,
                                MenuButton(button.action),
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::axes(Val::Px(18.0), Val::Px(12.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.22, 0.31, 0.47)),
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new(button.label),
                                    TextFont {
                                        font_size: 22.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                });
        });
}

fn pause_prompt(keyboard: Res<ButtonInput<KeyCode>>, mut next_pause: ResMut<NextState<Pause>>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_pause.set(Pause(true));
    }
}

fn update_button_visuals(
    mut buttons: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut color) in &mut buttons {
        *color = match *interaction {
            Interaction::Pressed => BackgroundColor(Color::srgb(0.45, 0.58, 0.78)),
            Interaction::Hovered => BackgroundColor(Color::srgb(0.31, 0.42, 0.62)),
            Interaction::None => BackgroundColor(Color::srgb(0.22, 0.31, 0.47)),
        };
    }
}

fn handle_menu_buttons(
    buttons: Query<(&Interaction, &MenuButton), (Changed<Interaction>, With<Button>)>,
    current_screen: Res<State<Screen>>,
    pause: Res<State<Pause>>,
    mut settings_return: ResMut<SettingsReturn>,
    profile: Res<LocalProfile>,
    config: Res<ConfigResource>,
    lobby: Res<LobbyReplica>,
    mut network_client: ResMut<NetworkClientResource>,
    mut network_state: ResMut<NetworkState>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_pause: ResMut<NextState<Pause>>,
    mut intents: MessageWriter<ClientIntent>,
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, button) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match button.0 {
            MenuButtonAction::Connect => {
                if let Err(error) =
                    connect_default(&mut network_client, &mut network_state, &config, &profile)
                {
                    eprintln!("Connect failed: {error}");
                } else {
                    next_screen.set(Screen::Lobby);
                }
            }
            MenuButtonAction::OpenSettings => {
                settings_return.screen = current_screen.get().to_owned();
                settings_return.pause = *pause.get();
                next_screen.set(Screen::Settings);
            }
            MenuButtonAction::BackFromSettings => {
                next_screen.set(settings_return.screen);
                next_pause.set(settings_return.pause);
            }
            MenuButtonAction::ToggleReady => {
                let any_unready = lobby.players.values().any(|player| !player.is_ready);
                intents.write(ClientIntent::SetReady(any_unready));
            }
            MenuButtonAction::StartGame => {
                intents.write(ClientIntent::StartGame);
            }
            MenuButtonAction::DisconnectToTitle => {
                disconnect(&mut network_client, &mut network_state);
                next_pause.set(Pause(false));
                next_screen.set(Screen::Title);
            }
            MenuButtonAction::ResumeGameplay => {
                next_pause.set(Pause(false));
            }
            MenuButtonAction::QuitToTitle => {
                next_pause.set(Pause(false));
                next_screen.set(Screen::Title);
            }
            MenuButtonAction::QuitApp => {
                exit.write(AppExit::Success);
            }
        }
    }
}
