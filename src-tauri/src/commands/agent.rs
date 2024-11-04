use tauri::State;
use tokio::sync::Mutex;
use crate::prelude::*;

#[tauri::command(async)]
pub async fn get_agent_list(
    state: State<'_, Mutex<AppStateAgents>>,
) -> EResult<CommandListAgents> {
    let state = state.lock().await;
    
    let agents = state.agents.iter().map(|agent| {
        agent.representation()
    }).collect();
    
    Ok(CommandListAgents {
        message: Some("Success".to_string()),
        agents,
    })
}