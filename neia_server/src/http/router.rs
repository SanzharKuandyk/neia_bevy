use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::http::{HeaderValue, header};
use axum::routing::post;
use axum::{http::StatusCode, response::IntoResponse};
use tower::ServiceBuilder;
use tower_http::{
    limit::RequestBodyLimitLayer, set_header::SetResponseHeaderLayer, timeout::TimeoutLayer,
    trace::TraceLayer,
};

use crate::AppState;

use super::token_handler::token_handler;

pub fn create_router(state: Arc<AppState>) -> Router {
    let http = state.config.http_config();
    let server_header_value = HeaderValue::from_str(&http.header_value)
        .unwrap_or_else(|_| HeaderValue::from_static("neia"));

    Router::new()
        .route("/connect", post(token_handler))
        .route("/token", post(token_handler))
        .fallback(handler_404)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(SetResponseHeaderLayer::if_not_present(
                    header::SERVER,
                    server_header_value,
                ))
                .layer(RequestBodyLimitLayer::new(http.max_request_body_size))
                .layer(TimeoutLayer::with_status_code(
                    StatusCode::REQUEST_TIMEOUT,
                    Duration::from_millis(http.request_timeout_ms),
                )),
        )
        .with_state(state)
}

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}
