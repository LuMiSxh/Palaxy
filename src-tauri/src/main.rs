// Prevents an additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::agents::initialize_agents;
use crate::commands::{agent, converter};
use specta_typescript::Typescript;
use tauri::{Builder, Manager};
use tauri_specta::{collect_commands, Builder as SpectaBuilder};
use tokio::sync::Mutex;

mod agents;
mod collector;
mod commands;
mod generator;
mod prelude;

fn main() {
    let builder = SpectaBuilder::<tauri::Wry>::new()
        // Then register them (separated by a comma)
        .commands(collect_commands![
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
        ]);

    // Needed for usize etc. to work
    let mut ts = Typescript::default();
    ts = ts.bigint(specta_typescript::BigIntExportBehavior::Number);

    // #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(ts, "../src/bindings.ts")
        .expect("Failed to export typescript bindings");

    Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            app.manage(Mutex::new(prelude::AppStateConverter::default()));
            app.manage(Mutex::new(prelude::AppStateAgents {
                agents: initialize_agents(),
            }));
            Ok(())
        })
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
