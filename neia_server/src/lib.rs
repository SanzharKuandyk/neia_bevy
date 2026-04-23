use std::sync::Arc;

use neia_mod::ModCollection;
use neia_shared::game::config::GameConfig;
use neia_shared::server::registry::Registry;
use tokio::sync::broadcast;

pub mod app;
pub mod config;
pub mod http;
pub mod server;

pub use app::AppState;
pub use config::Config;

pub struct Server {
    state: Arc<AppState>,
    shutdown_tx: broadcast::Sender<()>,
}

impl Server {
    pub fn new(
        config: Config,
        game_config: GameConfig,
        loaded_mods: ModCollection,
        registry: Registry,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let state = AppState::new(config, game_config, loaded_mods, registry);
        let (shutdown_tx, _) = broadcast::channel::<()>(1);

        tracing::info!("Server initialization complete");
        Ok(Self { state, shutdown_tx })
    }

    pub async fn start(&self, with_ctrl_c: bool) {
        if with_ctrl_c {
            let shutdown_tx = self.shutdown_tx.clone();
            tokio::spawn(async move {
                if tokio::signal::ctrl_c().await.is_ok() {
                    tracing::info!("Shutdown signal received (Ctrl+C).");
                    let _ = shutdown_tx.send(());
                }
            });
        }

        let mut server_shutdown = self.shutdown_tx.subscribe();
        let mut http_shutdown = self.shutdown_tx.subscribe();

        let server_task = {
            let state = self.state.clone();
            tokio::spawn(server::run(state, async move {
                let _ = server_shutdown.recv().await;
            }))
        };

        let http_task = {
            let state = self.state.clone();
            tokio::spawn(async move {
                if let Err(error) = http::run(state, async move {
                    let _ = http_shutdown.recv().await;
                })
                .await
                {
                    tracing::error!("HTTP server error: {}", error);
                }
            })
        };

        let graceful_shutdown_wait_secs = self.state.config.shutdown_config().graceful_wait_secs;
        let mut stop_signal = self.shutdown_tx.subscribe();

        tokio::select! {
            res = server_task => {
                tracing::info!("Game server task completed");
                if let Err(error) = res {
                    tracing::error!("Server task failed: {error}");
                }
                let _ = self.shutdown_tx.send(());
            }
            res = http_task => {
                tracing::info!("HTTP server task completed");
                if let Err(error) = res {
                    tracing::error!("HTTP task failed: {error}");
                }
                let _ = self.shutdown_tx.send(());
            }
            _ = stop_signal.recv() => {
                tracing::info!("Stop signal received, shutting down tasks...");
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(
            graceful_shutdown_wait_secs,
        ))
        .await;
        tracing::info!("Server shutdown complete");
    }

    pub fn stop(&self) {
        let _ = self.shutdown_tx.send(());
    }

    pub fn app_state(&self) -> Arc<AppState> {
        self.state.clone()
    }

    pub async fn game_config(&self) -> GameConfig {
        self.state.game_config.read().await.clone()
    }
}
