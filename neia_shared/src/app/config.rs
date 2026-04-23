use serde::{Deserialize, Serialize};

use crate::game::lobby::LobbyConfig;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub app: AppRuntimeConfig,
    pub network: NetworkConfig,
    pub server: ServerRuntimeConfig,
    pub http: HttpRuntimeConfig,
    #[serde(default)]
    pub shutdown: ShutdownRuntimeConfig,
    #[serde(default)]
    pub lobby: LobbyConfig,
    #[serde(default)]
    pub client: ClientRuntimeConfig,
    #[serde(default)]
    pub mods: ModsConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppRuntimeConfig {
    #[serde(default = "default_app_env")]
    pub app_env: String,
    #[serde(default = "default_max_log_files")]
    pub max_log_files: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkConfig {
    pub protocol_id: u64,
    pub available_bytes_per_tick: u64,
    #[serde(default = "default_resend_time_ms")]
    pub resend_time_ms: u64,
    pub gameplay_channel_memory_bytes: usize,
    pub snapshots_channel_memory_bytes: usize,
    pub chat_channel_memory_bytes: usize,
    pub errors_channel_memory_bytes: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerRuntimeConfig {
    #[serde(default = "default_bind_ip")]
    pub bind_ip: String,
    pub port: u16,
    pub max_clients: usize,
    pub delta_time_ms: u64,
    pub token_expire_seconds: u64,
    pub timeout_seconds: i32,
    #[serde(default = "default_max_messages_per_tick")]
    pub max_messages_per_tick: usize,
    #[serde(default = "default_tick_latency_warn_ms")]
    pub tick_latency_warn_ms: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpRuntimeConfig {
    #[serde(default = "default_bind_ip")]
    pub bind_ip: String,
    pub port: u16,
    pub header_value: String,
    pub request_timeout_ms: u64,
    #[serde(default = "default_max_request_body_size")]
    pub max_request_body_size: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClientRuntimeConfig {
    #[serde(default = "default_player_name")]
    pub default_player_name: String,
    #[serde(default = "default_server_addr")]
    pub default_server_addr: String,
    #[serde(default)]
    pub auto_connect: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModsConfig {
    #[serde(default = "default_manifests_dir")]
    pub manifests_dir: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShutdownRuntimeConfig {
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    #[serde(default = "default_graceful_wait_secs")]
    pub graceful_wait_secs: u64,
}

impl Default for ClientRuntimeConfig {
    fn default() -> Self {
        Self {
            default_player_name: default_player_name(),
            default_server_addr: default_server_addr(),
            auto_connect: false,
        }
    }
}

impl Default for ModsConfig {
    fn default() -> Self {
        Self {
            manifests_dir: default_manifests_dir(),
        }
    }
}

impl Default for ShutdownRuntimeConfig {
    fn default() -> Self {
        Self {
            timeout_secs: default_timeout_secs(),
            graceful_wait_secs: default_graceful_wait_secs(),
        }
    }
}

fn default_app_env() -> String {
    "development".to_string()
}
fn default_max_log_files() -> usize {
    5
}
fn default_resend_time_ms() -> u64 {
    300
}
fn default_bind_ip() -> String {
    "127.0.0.1".to_string()
}
fn default_max_messages_per_tick() -> usize {
    100
}
fn default_tick_latency_warn_ms() -> u64 {
    10
}
fn default_max_request_body_size() -> usize {
    16384
}
fn default_player_name() -> String {
    "Player".to_string()
}
fn default_server_addr() -> String {
    "127.0.0.1:5000".to_string()
}
fn default_manifests_dir() -> String {
    "mods".to_string()
}
fn default_timeout_secs() -> u64 {
    30
}
fn default_graceful_wait_secs() -> u64 {
    5
}
