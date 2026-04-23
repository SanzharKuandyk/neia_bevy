use std::path::PathBuf;

use clap::Parser;
use neia_config::LoadedConfigs;
use neia_shared::ServerError;
use neia_shared::app::{AppConfig, HttpRuntimeConfig, ServerRuntimeConfig, ShutdownRuntimeConfig};
use neia_shared::game::lobby::LobbyConfig;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub mod metrics;

#[derive(Debug, Default, Parser, Clone)]
pub struct Args {
    #[arg(long, action)]
    pub no_auth: bool,
    #[arg(long, action)]
    pub no_state: bool,
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub args: Args,
    pub app: AppConfig,
    pub private_key: [u8; 32],
}

impl Config {
    pub fn new() -> Result<(Self, neia_shared::game::config::GameConfig), ServerError> {
        let args = Args::parse();
        let loaded = LoadedConfigs::load_from_workspace_root(&args.root)
            .map_err(|err| ServerError::InitializationError(Some(err.into())))?;
        init_tracing(&loaded.app)?;
        Ok((
            Self {
                args,
                app: loaded.app,
                private_key: rand::random(),
            },
            loaded.game,
        ))
    }

    pub fn http_config(&self) -> &HttpRuntimeConfig {
        &self.app.http
    }

    pub fn server_config(&self) -> &ServerRuntimeConfig {
        &self.app.server
    }

    pub fn shutdown_config(&self) -> &ShutdownRuntimeConfig {
        &self.app.shutdown
    }

    pub fn lobby_config(&self) -> &LobbyConfig {
        &self.app.lobby
    }
}

fn init_tracing(app: &AppConfig) -> Result<(), ServerError> {
    let server_log_dir = "logs/server";
    let _ = std::fs::create_dir_all(server_log_dir);

    let file_appender: RollingFileAppender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("server")
        .filename_suffix("log")
        .max_log_files(app.app.max_log_files)
        .build(server_log_dir)
        .map_err(|err| {
            ServerError::InitializationError(Some(
                format!("Failed to create file appender for logs: {err}").into(),
            ))
        })?;
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    let log_level = match app.app.app_env.as_str() {
        "development" | "dev" => "debug",
        "production" | "prod" => "info",
        _ => "info",
    };

    let console_layer = fmt::layer().pretty().with_target(true);
    let file_layer = fmt::layer().with_ansi(false).with_writer(file_writer);

    tracing_subscriber::registry()
        .with(EnvFilter::new(log_level))
        .with(console_layer)
        .with(file_layer)
        .try_init()
        .ok();

    tracing::info!("App starting in {} mode", app.app.app_env);
    Ok(())
}
