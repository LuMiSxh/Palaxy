// Prevents an additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::agent::indexer::index_agents;
use crate::agent::injector::inject_functions;
use crate::commands::*;
use crate::prelude::AppStateLua;
use tauri::path::BaseDirectory;
use tauri::{Builder, Manager};
use tokio::sync::Mutex;

mod collector;
mod commands;
mod generator;
mod prelude;
mod agent;

fn main() {
    Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let agents_path = app.path().resolve("resources/agents", BaseDirectory::Resource)?;

            let lua = mlua::Lua::new();
            inject_functions(&lua)?;

            let agents = match index_agents(&agents_path, &lua) {
                Ok(agents) => agents,
                Err(e) => {
                    eprintln!("Error indexing agents: {:?}", e);
                    return Err(e.into());
                }
            };

            let lua_state = AppStateLua {
                source: agents_path,
                lua,
                agents,
            };

            app.manage(Mutex::new(lua_state));
            app.manage(Mutex::new(prelude::AppStateConverter::default()));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // setter
            set_source,
            set_bundle_flag,
            set_data,
            set_volume_sizes,
            // getter
            get_data,
            // reset
            reset,
            // processes
            analyze,
            bundle,
            convert,
            // agents
            test_agents,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
