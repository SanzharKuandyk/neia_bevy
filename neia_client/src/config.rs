use std::net::SocketAddr;

use neia_shared::NetworkConfig;
use neia_shared::UserData;

#[derive(Clone)]
pub struct ClientConfig {
    pub network: NetworkConfig,
    pub user_data: UserData,
}

#[derive(Debug, Clone)]
pub enum ConnectTarget {
    Remote(RemoteConnectInfo),
    Local(LocalConnectInfo),
}

#[derive(Debug, Clone)]
pub enum RemoteConnectInfo {
    Unsecure {
        server_addr: SocketAddr,
        protocol_id: u64,
    },
}

#[derive(Debug, Clone)]
pub struct LocalConnectInfo {
    pub server_addr: SocketAddr,
    pub protocol_id: u64,
}
