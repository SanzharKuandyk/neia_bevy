use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use renet::RenetServer;
use renet_netcode::NetcodeServerTransport;
use tokio::sync::Mutex;
use tokio::time::Interval;

use crate::AppState;
use crate::config::metrics::{
    record_message_handling_latency, record_network_metrics, record_tick_latency,
};

pub mod handlers;
pub mod sender;
pub mod session;
pub mod state;
pub mod transport;

pub use sender::*;
pub use session::SessionState;
pub use state::ServerState;

use self::handlers::events::handle_events;
use self::handlers::messages::handle_messages;

pub async fn run(app_state: Arc<AppState>, shutdown: impl Future<Output = ()>) {
    let server_ip: IpAddr = app_state
        .config
        .server_config()
        .bind_ip
        .parse()
        .expect("valid server bind ip");
    let (transport_server, mut transport) = transport::build_transport(
        &app_state.config.app.network,
        app_state.config.private_key,
        server_ip,
        app_state.config.server_config().port,
        app_state.config.server_config().max_clients,
        app_state.config.args.no_auth,
    );
    let server = Arc::new(Mutex::new(transport_server));

    tracing::info!(
        "Game server started on {}:{}",
        app_state.config.server_config().bind_ip,
        app_state.config.server_config().port
    );

    let delta_time = Duration::from_millis(app_state.config.server_config().delta_time_ms);
    let interval = tokio::time::interval(delta_time);

    tokio::select! {
        _ = tick_loop(interval, server, delta_time, &mut transport, app_state.clone()) => {},
        _ = shutdown => tracing::info!("Game server shutting down..."),
    }

    {
        let game_config = app_state.get_game_config().await;
        let lobby_config = app_state.config.lobby_config().clone();
        let mut session_guard = app_state.server_state.session.write().await;
        *session_guard = SessionState::new_lobby(game_config, lobby_config);
    }

    tracing::debug!("Session reset to lobby during server shutdown");
    tracing::info!("Game server has shut down.");
}

async fn tick_loop(
    mut interval: Interval,
    server: Arc<Mutex<RenetServer>>,
    delta_time: Duration,
    transport: &mut NetcodeServerTransport,
    app_state: Arc<AppState>,
) {
    let mut tick: u64 = 0;
    let max_messages_per_tick = app_state.config.server_config().max_messages_per_tick;
    let tick_latency_warn_ms = app_state.config.server_config().tick_latency_warn_ms;

    loop {
        interval.tick().await;
        tick += 1;
        let tick_start = Instant::now();

        let msg_start = Instant::now();
        {
            let mut server = server.lock().await;

            server.update(delta_time);

            if let Err(error) = transport.update(delta_time, &mut server) {
                tracing::error!("Transport update failed: {}", error);
            }

            record_network_metrics(&server);
            handle_events(&mut server, transport, app_state.clone(), tick).await;
        }

        let mut message_deltas = {
            let mut server = server.lock().await;
            handle_messages(&mut server, app_state.clone(), tick, max_messages_per_tick).await
        };

        record_tick_latency(tick, tick_start.elapsed(), tick_latency_warn_ms);
        record_message_handling_latency(msg_start.elapsed());

        let mut session_deltas = {
            let dt_secs = delta_time.as_secs_f32();
            let mut session_guard = app_state.get_server_state().get_session_state_mut().await;
            session_guard.tick(dt_secs, tick)
        };

        let mut all_deltas = Vec::new();
        all_deltas.append(&mut message_deltas);
        all_deltas.append(&mut session_deltas);
        all_deltas.sort_by_key(|delta| delta.priority);

        {
            let mut server = server.lock().await;

            if !all_deltas.is_empty()
                && let Err(error) = broadcast_deltas(
                    &mut server,
                    tick,
                    all_deltas,
                    neia_shared::server::channel::ServerChannel::Gameplay,
                )
            {
                tracing::error!("Failed to broadcast deltas: {}", error);
            }

            transport.send_packets(&mut server);
        }
    }
}
