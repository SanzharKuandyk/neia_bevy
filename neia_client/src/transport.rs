use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;

use neia_shared::NetworkConfig;
use neia_shared::UserData;
use neia_shared::server::channel::ServerChannel;
use renet::RenetClient;
use renet_netcode::{ClientAuthentication, NETCODE_USER_DATA_BYTES, NetcodeClientTransport};

use crate::config::{ConnectTarget, RemoteConnectInfo};
use crate::error::ClientError;

pub(crate) fn bind_socket() -> Result<UdpSocket, ClientError> {
    let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);
    UdpSocket::bind(bind_addr).map_err(|err| ClientError::Network(err.to_string()))
}

pub(crate) fn build_authentication(
    target: ConnectTarget,
    user_data: &UserData,
) -> Result<ClientAuthentication, ClientError> {
    match target {
        ConnectTarget::Local(info) => Ok(ClientAuthentication::Unsecure {
            protocol_id: info.protocol_id,
            client_id: rand::random::<u64>(),
            server_addr: info.server_addr,
            user_data: Some(encode_user_data(user_data)?),
        }),
        ConnectTarget::Remote(RemoteConnectInfo::Unsecure {
            server_addr,
            protocol_id,
        }) => Ok(ClientAuthentication::Unsecure {
            protocol_id,
            client_id: rand::random::<u64>(),
            server_addr,
            user_data: Some(encode_user_data(user_data)?),
        }),
    }
}

pub(crate) fn build_transport(
    network: NetworkConfig,
    authentication: ClientAuthentication,
    socket: UdpSocket,
    current_time: std::time::Duration,
) -> Result<(RenetClient, NetcodeClientTransport), ClientError> {
    let client = RenetClient::new(connection_config(&network));
    let transport = NetcodeClientTransport::new(current_time, authentication, socket)
        .map_err(|err| ClientError::Network(err.to_string()))?;
    Ok((client, transport))
}

fn connection_config(network: &NetworkConfig) -> renet::ConnectionConfig {
    use renet::{ChannelConfig, ConnectionConfig, SendType};

    let channels = vec![
        ChannelConfig {
            channel_id: ServerChannel::Gameplay.into(),
            max_memory_usage_bytes: network.gameplay_channel_memory_bytes,
            send_type: SendType::ReliableOrdered {
                resend_time: Duration::from_millis(network.resend_time_ms),
            },
        },
        ChannelConfig {
            channel_id: ServerChannel::Snapshots.into(),
            max_memory_usage_bytes: network.snapshots_channel_memory_bytes,
            send_type: SendType::ReliableUnordered {
                resend_time: Duration::from_millis(network.resend_time_ms),
            },
        },
        ChannelConfig {
            channel_id: ServerChannel::Chat.into(),
            max_memory_usage_bytes: network.chat_channel_memory_bytes,
            send_type: SendType::Unreliable,
        },
        ChannelConfig {
            channel_id: ServerChannel::Errors.into(),
            max_memory_usage_bytes: network.errors_channel_memory_bytes,
            send_type: SendType::Unreliable,
        },
    ];

    ConnectionConfig {
        available_bytes_per_tick: network.available_bytes_per_tick,
        server_channels_config: channels.clone(),
        client_channels_config: channels,
    }
}

fn encode_user_data(user_data: &UserData) -> Result<[u8; NETCODE_USER_DATA_BYTES], ClientError> {
    let bytes = postcard::to_allocvec(user_data)
        .map_err(|err| ClientError::InvalidUserData(err.to_string()))?;
    if bytes.len() > NETCODE_USER_DATA_BYTES {
        return Err(ClientError::InvalidUserData(
            "Encoded user_data exceeds netcode limit".into(),
        ));
    }

    let mut buffer = [0u8; NETCODE_USER_DATA_BYTES];
    buffer[..bytes.len()].copy_from_slice(&bytes);
    Ok(buffer)
}
