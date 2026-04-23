use std::time::Duration;

use metrics::{counter, gauge, histogram};
//use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use renet::RenetServer;

// Global Prometheus handle to expose via HTTP
//pub static PROM_HANDLE: LazyLock<PrometheusHandle> = LazyLock::new(|| {
//    PrometheusBuilder::new()
//        .install_recorder()
//        .expect("failed to install Prometheus recorder")
//});

/// Record active core tick latency (excluding interval wait/sleep) in ms and warn if slow.
pub fn record_tick_latency(tick: u64, elapsed: Duration, warn_threshold_ms: u64) {
    let ms = elapsed.as_secs_f64() * 1000.0;
    histogram!("server_core_tick_time_ms").record(ms);
    gauge!("server_core_tick_time_latest_ms").set(ms);
    if ms as u64 > warn_threshold_ms {
        tracing::warn!(
            tick,
            ms,
            "Slow active core tick above {} ms",
            warn_threshold_ms
        );
    }
}

/// Record message handling latency in ms.
pub fn record_message_handling_latency(elapsed: Duration) {
    let ms = elapsed.as_secs_f64() * 1000.0;
    histogram!("server_message_handling_time_ms").record(ms);
}

/// Per-client network stats
pub fn record_network_metrics(server: &RenetServer) {
    let mut total_bytes_sent = 0.0;
    let mut total_bytes_received = 0.0;
    let clients = server.connected_clients();

    for client_id in server.clients_id_iter() {
        let rtt = server.rtt(client_id);
        let packet_loss = server.packet_loss(client_id);
        let bytes_sent = server.bytes_sent_per_sec(client_id);
        let bytes_received = server.bytes_received_per_sec(client_id);

        total_bytes_sent += bytes_sent;
        total_bytes_received += bytes_received;

        let client_label = client_id.to_string();

        gauge!("client_rtt_seconds", "client_id" => client_label.clone()).set(rtt / 1000.0);
        gauge!("client_packet_loss_ratio", "client_id" => client_label.clone()).set(packet_loss);
        gauge!("client_bytes_sent_per_sec", "client_id" => client_label.clone()).set(bytes_sent);
        gauge!("client_bytes_received_per_sec", "client_id" => client_label.clone())
            .set(bytes_received);
    }

    counter!("server_packets_sent_total").increment(total_bytes_sent as u64);
    counter!("server_packets_received_total").increment(total_bytes_received as u64);
    gauge!("server_connected_clients").set(clients as f64);
}

/// Increment connect/disconnect counters
pub fn record_client_connected() {
    counter!("server_client_connect_total").increment(1);
}

pub fn record_client_disconnected() {
    counter!("server_client_disconnect_total").increment(1);
}

// HTTP endpoint for Prometheus scrape.
// Example route: GET /metrics
//pub async fn metrics_endpoint() -> axum::response::Html<String> {
//    axum::response::Html(PROM_HANDLE.render())
//}
