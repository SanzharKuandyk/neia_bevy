use neia_defaults::deltas::team_added::TeamAdded;
use neia_shared::ServerError;
use neia_shared::server::DeltaData;
use neia_shared::server::channel::ServerChannel;
use neia_shared::server::message::MessageTarget;
use neia_shared::server::response::ServerResponse;
use renet::RenetServer;

use crate::server::sender::send_message;
use crate::server::state::ServerState;

pub async fn handle_add_team(
    server: &mut RenetServer,
    server_state: &ServerState,
    team_name: String,
    tick: u64,
) -> Result<u8, ServerError> {
    let team_id = {
        let mut session_guard = server_state.session.write().await;
        let lobby = session_guard.as_lobby_mut()?;
        let team_id = lobby.add_team(team_name.clone())?;

        tracing::info!(
            "Team '{}' added with ID {}",
            lobby.teams[&team_id].name,
            team_id
        );

        team_id
    };

    // Broadcast TeamAdded delta
    let delta = TeamAdded {
        team_id,
        name: team_name,
    };
    if let Ok(delta_data) = DeltaData::from_delta(&delta) {
        let response = ServerResponse {
            tick,
            deltas: vec![delta_data],
            error: None,
        };
        let _ = send_message(
            server,
            MessageTarget::All,
            response,
            ServerChannel::Gameplay,
            None,
        )
        .await;
    }

    Ok(team_id)
}
