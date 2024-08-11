// Prevents an additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::commands::{flow_analyze, flow_convert, flow_volume};

mod collector;
mod commands;
mod generator;
mod prelude;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            flow_convert,
            flow_analyze,
            flow_volume,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
