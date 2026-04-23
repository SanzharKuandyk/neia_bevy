use bevy::prelude::*;
use neia_defaults::messages::ready::Ready;
use neia_defaults::messages::start_game::StartGame;

use crate::net::{NetworkClientResource, send_system_message};
use crate::state::Screen;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ClientIntent>()
            .add_message::<LobbyChanged>()
            .add_message::<GameplayChanged>()
            .add_systems(Update, route_client_intents.run_if(in_state(Screen::Lobby)));
    }
}

#[derive(Message, Debug, Clone)]
pub enum ClientIntent {
    SetReady(bool),
    StartGame,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct LobbyChanged;

#[derive(Message, Debug, Clone, Copy)]
pub struct GameplayChanged;

fn route_client_intents(
    mut intents: MessageReader<ClientIntent>,
    mut network_client: ResMut<NetworkClientResource>,
) {
    let Some(client) = network_client.client.as_mut() else {
        return;
    };

    for intent in intents.read() {
        match intent {
            ClientIntent::SetReady(ready) => {
                let _ = send_system_message(client, &Ready { ready: *ready });
            }
            ClientIntent::StartGame => {
                let _ = send_system_message(client, &StartGame);
            }
        }
    }
}
