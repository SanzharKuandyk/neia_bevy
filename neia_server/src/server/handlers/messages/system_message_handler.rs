use std::any::Any;
use std::sync::Arc;

use neia_defaults::messages::add_team::AddTeam;
use neia_defaults::messages::ready::Ready;
use neia_defaults::messages::start_game::StartGame;
use neia_shared::ServerError;
use renet::{ClientId, RenetServer};
use uuid::Uuid;

use crate::AppState;
use crate::server::sender::send_err_message;

pub mod add_team;
pub mod ready;
pub mod start_game;

use self::add_team::handle_add_team;
use self::ready::handle_ready_msg;
use self::start_game::handle_game_start;

pub async fn handle_sys_message(
    msg_any: Box<dyn Any + Send>,
    server: &mut RenetServer,
    app_state: &Arc<AppState>,
    client_id: ClientId,
    player_id: Uuid,
    tick: u64,
    delta_secs: f32,
) {
    if msg_any.is::<StartGame>() {
        handle_game_start(server, app_state, client_id, tick, delta_secs).await;
    } else if let Some(msg) = msg_any.downcast_ref::<Ready>() {
        handle_ready_msg(server, &app_state.server_state, player_id, msg.ready, tick).await;
    } else if let Some(msg) = msg_any.downcast_ref::<AddTeam>() {
        match handle_add_team(server, &app_state.server_state, msg.name.clone(), tick).await {
            Ok(team_id) => {
                tracing::debug!("Team added successfully with ID: {}", team_id);
            }
            Err(e) => {
                tracing::warn!("Failed to add team: {}", e);
                let _ = send_err_message(server, client_id, e, tick).await;
            }
        }
    } else {
        let err = ServerError::InvalidMessage(Some(
            "System message type downcast failed in handle_sys_message".into(),
        ));
        let _ = send_err_message(server, client_id, err, tick).await;
    }
}
