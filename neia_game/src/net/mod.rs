use bevy::prelude::*;
use neia_client::{
    Client, ClientConfig, ClientEvent, ClientStatus, ConnectTarget, IncomingBatch,
    RemoteConnectInfo,
};
use neia_defaults::deltas::game_started::GameStarted;
use neia_defaults::deltas::player_joined_lobby::PlayerJoinedLobby;
use neia_defaults::deltas::player_ready_changed::PlayerReadyChanged;
use neia_shared::client::message::Message;
use neia_shared::server::delta::Delta;

use crate::client::{GameplayChanged, LobbyChanged};
use crate::state::{
    ConfigResource, GameSession, LobbyPlayerView, LobbyReplica, LocalProfile, NetworkState, Screen,
};

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetworkClientResource>()
            .init_resource::<NetworkState>()
            .init_resource::<LobbyReplica>()
            .init_resource::<GameSession>()
            .add_message::<ClientUpdate>()
            .add_systems(Update, poll_network_client)
            .add_systems(Update, apply_incoming_batches);
    }
}

#[derive(Resource, Default)]
pub struct NetworkClientResource {
    pub client: Option<Client>,
}

#[derive(Message, Debug)]
pub struct ClientUpdate(pub IncomingBatch);

pub fn connect_default(
    network_client: &mut NetworkClientResource,
    network_state: &mut NetworkState,
    config: &ConfigResource,
    profile: &LocalProfile,
) -> Result<(), String> {
    let server_addr = config
        .app
        .client
        .default_server_addr
        .parse()
        .map_err(|err| format!("invalid client.default_server_addr: {err}"))?;
    let client = Client::connect(
        ClientConfig {
            network: config.app.network.clone(),
            user_data: profile.user_data.clone(),
        },
        ConnectTarget::Remote(RemoteConnectInfo::Unsecure {
            server_addr,
            protocol_id: config.app.network.protocol_id,
        }),
    )
    .map_err(|err| err.to_string())?;

    network_client.client = Some(client);
    network_state.status = ClientStatus::Connecting;
    network_state.connected = true;
    Ok(())
}

pub fn disconnect(network_client: &mut NetworkClientResource, network_state: &mut NetworkState) {
    if let Some(client) = network_client.client.as_mut() {
        let _ = client.disconnect();
    }
    network_client.client = None;
    network_state.status = ClientStatus::Disconnected;
    network_state.connected = false;
}

pub fn send_system_message<M: Message>(client: &mut Client, message: &M) -> Result<(), String> {
    client
        .send_system_message(message)
        .map_err(|err| err.to_string())
}

fn poll_network_client(
    mut network_client: ResMut<NetworkClientResource>,
    mut network_state: ResMut<NetworkState>,
    mut updates: MessageWriter<ClientUpdate>,
) {
    let Some(client) = network_client.client.as_mut() else {
        return;
    };

    let (events, batches) = client.poll_events();
    for event in events {
        match event {
            ClientEvent::StatusChanged(status) => {
                network_state.status = status;
            }
            ClientEvent::Error(error) => {
                network_state.status = ClientStatus::ConnectionError(error.to_string());
                eprintln!("network error: {error}");
            }
        }
    }

    for batch in batches {
        updates.write(ClientUpdate(batch));
    }
}

fn apply_incoming_batches(
    mut updates: MessageReader<ClientUpdate>,
    mut lobby: ResMut<LobbyReplica>,
    mut game_session: ResMut<GameSession>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut lobby_changed: MessageWriter<LobbyChanged>,
    mut gameplay_changed: MessageWriter<GameplayChanged>,
) {
    for update in updates.read() {
        let batch = &update.0;
        let mut lobby_dirty = false;
        let mut gameplay_dirty = false;
        game_session.last_tick = batch.tick;

        for delta in &batch.deltas {
            match delta.delta_type.as_str() {
                "sys.player_joined_lobby" => {
                    if let Ok(delta) = PlayerJoinedLobby::from_bytes(&delta.data) {
                        lobby.players.insert(
                            delta.player_id,
                            LobbyPlayerView {
                                id: delta.player_id,
                                name: delta.name,
                                is_ready: false,
                            },
                        );
                        lobby_dirty = true;
                    }
                }
                "sys.player_ready_changed" => {
                    if let Ok(delta) = PlayerReadyChanged::from_bytes(&delta.data) {
                        if let Some(player) = lobby.players.get_mut(&delta.player_id) {
                            player.is_ready = delta.is_ready;
                        }
                        lobby_dirty = true;
                    }
                }
                "sys.game_started" => {
                    if GameStarted::from_bytes(&delta.data).is_ok() {
                        game_session.started = true;
                        next_screen.set(Screen::Gameplay);
                        gameplay_dirty = true;
                    }
                }
                _ => {}
            }
        }

        if lobby_dirty {
            lobby_changed.write(LobbyChanged);
        }
        if gameplay_dirty {
            gameplay_changed.write(GameplayChanged);
        }
    }
}
