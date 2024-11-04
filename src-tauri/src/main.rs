// Prevents an additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::commands::{converter, agent};
use tauri::{Builder, Manager};
use tokio::sync::Mutex;
use crate::agents::initialize_agents;

mod collector;
mod commands;
mod generator;
mod prelude;
mod agents;

fn main() {
    Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            app.manage(Mutex::new(prelude::AppStateConverter::default()));
            app.manage(Mutex::new(prelude::AppStateAgents {
                agents: initialize_agents(),
            }));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            converter::set_source,
            converter::set_bundle_flag,
            converter::set_data,
            converter::set_volume_sizes,
            converter::get_data,
            converter::reset,
            converter::analyze,
            converter::bundle,
            converter::convert,
            //
            agent::get_agent_list,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
