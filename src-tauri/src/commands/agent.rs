use crate::prelude::*;
use crate::types::{AgentMeta, CommListAgents};
use tauri::State;
use tokio::sync::Mutex;

#[tauri::command(async)]
#[specta::specta]
pub async fn get_agent_list(state: State<'_, Mutex<AgentState>>) -> EResult<CommListAgents> {
    let start = std::time::Instant::now();

    let state = state.lock().await;

    let agents: Vec<AgentMeta> = state
        .agents
        .iter()
        .map(|agent| agent.representation())
        .collect();

    Ok(CommListAgents {
        duration: start.elapsed().as_secs(),
        comment: None,
        payload: Some(agents),
    })
}
