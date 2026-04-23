use std::net::{IpAddr, SocketAddr};

use tokio::net::TcpListener;

use crate::AppState;

pub mod router;
pub mod token_handler;

pub async fn run(
    app_state: std::sync::Arc<AppState>,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<(), Box<dyn std::error::Error>> {
    let http = app_state.config.http_config();
    let bind_ip: IpAddr = http.bind_ip.parse()?;
    let addr = SocketAddr::new(bind_ip, http.port);
    let listener = TcpListener::bind(addr).await?;
    let app = router::create_router(app_state);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await?;

    Ok(())
}
