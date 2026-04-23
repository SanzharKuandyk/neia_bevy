use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::StatusCode;
use axum::{Json, extract::State, response::IntoResponse};
use neia_shared::UserData;
use renet_netcode::ConnectToken;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use crate::server::session::SessionState;

#[derive(Deserialize)]
pub struct TokenRequest {
    pub id: Uuid,
    pub team_id: Option<u8>,
    pub name: String,
}

impl TokenRequest {
    fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Player name cannot be empty".to_string());
        }
        if self.name.len() > 64 {
            return Err("Player name too long (max 64 chars)".to_string());
        }
        if self.name.chars().any(|c| c.is_control()) {
            return Err("Player name contains invalid characters".to_string());
        }
        Ok(())
    }
}

#[derive(Serialize)]
pub struct TokenResponse {
    token: String,
}

pub async fn token_handler(
    State(app_state): State<Arc<AppState>>,
    Json(req): Json<TokenRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    if let Err(error) = req.validate() {
        tracing::warn!("Invalid token request: {}", error);
        return Err(StatusCode::BAD_REQUEST);
    }

    {
        let session_guard = app_state.server_state.session.read().await;
        match &*session_guard {
            SessionState::Lobby(lobby) => {
                if lobby.player_count() >= app_state.config.lobby_config().max_players {
                    tracing::warn!("Token request rejected: lobby is full");
                    return Err(StatusCode::CONFLICT);
                }

                if app_state.config.lobby_config().teams_enabled {
                    match req.team_id {
                        Some(tid) if !lobby.teams.contains_key(&tid) => {
                            tracing::warn!("Token request rejected: team {} not found", tid);
                            return Err(StatusCode::BAD_REQUEST);
                        }
                        None => {
                            tracing::warn!("Token request rejected: team_id required");
                            return Err(StatusCode::BAD_REQUEST);
                        }
                        _ => {}
                    }
                }
            }
            SessionState::InGame(_) => {}
            SessionState::Finished { .. } => {
                tracing::warn!("Token request rejected: game is finished");
                return Err(StatusCode::CONFLICT);
            }
        }
    }

    {
        let mut owner_info = app_state.server_state.owner_info.lock().await;
        if owner_info.cid.is_none() {
            owner_info.player_id = Some(req.id);
            tracing::info!(
                player_id = %req.id,
                player_name = %req.name,
                "Assigned player as game owner"
            );
        }
    }

    let network = &app_state.config.app.network;
    let server = app_state.config.server_config();

    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| {
        tracing::error!("System time error during token generation");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let server_ip: IpAddr = server
        .bind_ip
        .parse()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let server_addr = SocketAddr::new(server_ip, server.port);

    let client_id = rand::random::<u64>();
    let private_key = &app_state.config.private_key;

    let safe_name: String = req.name.chars().take(64).collect();
    let user_data_obj = UserData {
        id: req.id,
        name: safe_name,
        team_id: req.team_id,
        role_id: None,
    };

    let serialized = postcard::to_allocvec(&user_data_obj).map_err(|error| {
        tracing::error!("Failed to encode user data: {}", error);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut user_data = [0u8; renet_netcode::NETCODE_USER_DATA_BYTES];
    user_data[..serialized.len()].copy_from_slice(&serialized);

    let token = ConnectToken::generate(
        current_time,
        network.protocol_id,
        server.token_expire_seconds,
        client_id,
        server.timeout_seconds,
        vec![server_addr],
        Some(&user_data),
        private_key,
    )
    .map_err(|error| {
        tracing::error!("Failed to generate token: {}", error);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut buf = Vec::new();
    token.write(&mut buf).map_err(|error| {
        tracing::error!("Failed to write token: {}", error);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!(
        player_id = %req.id,
        player_name = %req.name,
        client_id = %client_id,
        "Token generated successfully"
    );

    Ok(Json(TokenResponse {
        token: hex::encode(&buf),
    }))
}
