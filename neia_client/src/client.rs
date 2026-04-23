use std::time::{Instant, SystemTime, UNIX_EPOCH};

use neia_shared::UserData;
use neia_shared::client::message::{Message, encode_message};
use neia_shared::server::channel::ServerChannel;
use renet::RenetClient;
use renet_netcode::NetcodeClientTransport;

use crate::config::{ClientConfig, ConnectTarget};
use crate::error::{ClientError, ClientStatus};
use crate::events::{ClientEvent, IncomingBatch};
use crate::{poll, transport};

pub struct Client {
    user_data: UserData,
    client: RenetClient,
    transport: NetcodeClientTransport,
    last_poll: Instant,
    last_status: ClientStatus,
}

impl Client {
    pub fn connect(config: ClientConfig, target: ConnectTarget) -> Result<Self, ClientError> {
        let ClientConfig { network, user_data } = config;
        let socket = transport::bind_socket()?;
        let authentication = transport::build_authentication(target, &user_data)?;
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| ClientError::Network(err.to_string()))?;
        let (client, transport) =
            transport::build_transport(network, authentication, socket, current_time)?;

        Ok(Self {
            user_data,
            client,
            transport,
            last_poll: Instant::now(),
            last_status: ClientStatus::Connecting,
        })
    }

    pub fn disconnect(&mut self) -> Result<(), ClientError> {
        self.client.disconnect();
        self.transport.disconnect();
        self.last_status = ClientStatus::Disconnected;
        Ok(())
    }

    pub fn user_data(&self) -> &UserData {
        &self.user_data
    }

    pub fn poll_events(&mut self) -> (Vec<ClientEvent>, Vec<IncomingBatch>) {
        poll::poll(
            &mut self.client,
            &mut self.transport,
            &mut self.last_poll,
            &mut self.last_status,
        )
    }

    pub fn send_system_message<M: Message>(&mut self, msg: &M) -> Result<(), ClientError> {
        self.send_reliable(msg)
    }

    pub fn send_game_message<M: Message>(&mut self, msg: &M) -> Result<(), ClientError> {
        self.send_reliable(msg)
    }

    fn send_reliable<M: Message>(&mut self, msg: &M) -> Result<(), ClientError> {
        let bytes = encode_message(msg).map_err(|err| ClientError::Protocol(err.to_string()))?;
        self.client.send_message(ServerChannel::Gameplay, bytes);
        Ok(())
    }
}
