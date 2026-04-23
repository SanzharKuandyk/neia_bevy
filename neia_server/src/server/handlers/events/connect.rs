use neia_defaults::deltas::player_joined_lobby::PlayerJoinedLobby;
use neia_defaults::deltas::player_joined_team::PlayerJoinedTeam;
use neia_shared::game::lobby::LobbyState;
use neia_shared::server::DeltaData;
use neia_shared::server::error::ServerError;
use neia_shared::{Player, UserData, invalid_state};
use renet::RenetServer;

use crate::AppState;
use crate::server::sender::send_err_message;
use crate::server::session::{GameSession, SessionState};

use super::helpers::{broadcast_deltas, push_delta};

/// Handle player joining lobby.
fn handle_lobby_join(
    lobby: &mut LobbyState,
    user_data: UserData,
    deltas: &mut Vec<DeltaData>,
) -> Result<(), ServerError> {
    let player_id = user_data.id;
    let player_name = user_data.name.clone();
    let team_id = user_data.team_id;

    lobby
        .add_player(
            player_id,
            team_id,
            player_name.clone(),
            user_data.role_id.clone(),
        )
        .map_err(|e| invalid_state!(e.to_string()))?;

    tracing::info!(
        "Player {} ({}) joined lobby (team: {:?})",
        player_name,
        player_id,
        team_id
    );

    push_delta(
        deltas,
        PlayerJoinedLobby {
            player_id,
            name: player_name,
        },
    );

    // Broadcast team assignment delta if teams are enabled
    if let Some(tid) = team_id {
        push_delta(
            deltas,
            PlayerJoinedTeam {
                player_id,
                team_id: tid,
            },
        );
    }

    Ok(())
}

/// Handle player joining a running game (mid-game join).
fn handle_mid_game_join(
    game: &mut GameSession,
    user_data: UserData,
    _deltas: &mut Vec<DeltaData>,
) -> Result<(), ServerError> {
    let player_id = user_data.id;
    let player_name = user_data.name.clone();
    let team_id = user_data.team_id;

    let player = Player::new(player_id, user_data.name, team_id, None);
    game.state.add_player(player);

    tracing::info!("Player {} ({}) joined running game", player_name, player_id);

    // Game-specific spawning logic would go here (via a hook or callback)

    Ok(())
}

/// Main connection handler.
pub async fn handle_player_connect(
    server: &mut RenetServer,
    app_state: &AppState,
    client_id: u64,
    user_data: UserData,
    tick: u64,
) {
    let player_id = user_data.id;

    {
        let mut client_map = app_state.server_state.client_map.lock().await;
        client_map.insert(client_id, player_id);
    }

    let mut session_guard = app_state.server_state.session.write().await;
    let mut deltas = Vec::new();

    let result = match &mut *session_guard {
        SessionState::Lobby(lobby) => handle_lobby_join(lobby, user_data, &mut deltas),
        SessionState::InGame(game) => handle_mid_game_join(game, user_data, &mut deltas),
        SessionState::Finished { .. } => {
            tracing::warn!("Player tried to connect to finished game");
            drop(session_guard);

            let error = invalid_state!("Game is finished");
            let _ = send_err_message(server, client_id, error, tick).await;
            server.disconnect(client_id);
            return;
        }
    };

    drop(session_guard);

    if let Err(e) = result {
        tracing::error!("Failed to handle player connect: {}", e);
        let _ = send_err_message(server, client_id, e, tick).await;
        server.disconnect(client_id);
        return;
    }

    if let Err(e) = broadcast_deltas(server, tick, deltas).await {
        tracing::error!("Failed to broadcast connection deltas: {}", e);
    }
}
