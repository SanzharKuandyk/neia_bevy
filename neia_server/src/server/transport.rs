use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use neia_shared::NetworkConfig;
use neia_shared::server::channel::ServerChannel;
use renet::{ChannelConfig, ConnectionConfig, RenetServer, SendType};
use renet_netcode::{
    NetcodeServerTransport, ServerAuthentication, ServerConfig as NetcodeServerConfig,
};

pub fn build_transport(
    network: &NetworkConfig,
    private_key: [u8; 32],
    server_ip: IpAddr,
    server_port: u16,
    max_clients: usize,
    no_auth: bool,
) -> (RenetServer, NetcodeServerTransport) {
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

    let connection_cfg = ConnectionConfig {
        available_bytes_per_tick: network.available_bytes_per_tick,
        server_channels_config: channels.clone(),
        client_channels_config: channels,
    };
    let server = RenetServer::new(connection_cfg);

    let fallback_ip = if server_ip.is_unspecified() {
        IpAddr::V4(Ipv4Addr::UNSPECIFIED)
    } else {
        server_ip
    };
    let addr = SocketAddr::new(fallback_ip, server_port);
    let socket = UdpSocket::bind(addr).expect("Failed to bind UDP socket for RenetServer");

    let authentication = if no_auth {
        ServerAuthentication::Unsecure
    } else {
        ServerAuthentication::Secure { private_key }
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let net_cfg = NetcodeServerConfig {
        current_time: now,
        max_clients,
        protocol_id: network.protocol_id,
        public_addresses: vec![addr],
        authentication,
    };

    let transport = NetcodeServerTransport::new(net_cfg, socket)
        .expect("Failed to create NetcodeServerTransport");

    (server, transport)
}
