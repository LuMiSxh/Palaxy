// Prevents an additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::commands::*;
use tauri::{Builder, Manager};
use tokio::sync::Mutex;

mod collector;
mod commands;
mod generator;
mod prelude;

fn main() {
    Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            app.manage(Mutex::new(prelude::AppState::default()));
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
