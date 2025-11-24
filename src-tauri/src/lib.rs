//! Main application initialization module.
//!
//! This module is responsible for setting up and running the Tauri application.
//! It handles command registration, TypeScript bindings export, plugin setup,
//! and application state initialization.

use crate::commands::converter::{
    ConversionCompleteEvent, ConversionStartEvent, ImageProgressEvent, StatusMessageEvent,
    VolumeCompleteEvent, VolumeStartEvent, conv_analyze, conv_bundle, conv_convert, conv_state_get,
    conv_state_reset, conv_state_set,
};
use crate::commands::management::mgmt_get_logs_path;
use specta_typescript::Typescript;
use tauri::{Builder, Manager};
use tauri_specta::{Builder as SpectaBuilder, collect_commands, collect_events};
use tokio::sync::Mutex;

mod collector;
mod commands;
mod generator;
mod prelude;
mod types;
#[macro_use]
mod macros;

/// Initializes and runs the Tauri application.
///
/// This function performs the following tasks:
/// 1. Register commands for frontend-backend communication
/// 2. Export TypeScript bindings for development build
/// 3. Configure the application with necessary plugins
/// 4. Set up application state, including database initialization
/// 5. Start the Tauri application with the configured settings
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Register commands for the frontend to call
    let spectra_builder = SpectaBuilder::<tauri::Wry>::new()
        .commands(collect_commands![
            conv_state_set,
            conv_state_get,
            conv_state_reset,
            conv_analyze,
            conv_bundle,
            conv_convert,
            mgmt_get_logs_path
        ])
        .events(collect_events![
            VolumeStartEvent,
            VolumeCompleteEvent,
            ImageProgressEvent,
            ConversionStartEvent,
            ConversionCompleteEvent,
            StatusMessageEvent,
        ]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    {
        // Configure TypeScript bindings export for development
        let mut ts = Typescript::default();
        ts = ts.bigint(specta_typescript::BigIntExportBehavior::Number);

        spectra_builder
            .export(ts, "../src/types/bindings.ts")
            .expect("Failed to export typescript bindings");
    }

    Builder::default()
        // Ensure only one instance of the app runs at a time
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        // Configure application logging
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("logs".to_string()),
                    },
                ))
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
                .level(log::LevelFilter::Info)
                .format(|out, message, record| {
                    let time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                    out.finish(format_args!(
                        "[{} {} {}] [{}:{}] {}",
                        time,
                        record.level(),
                        record.target(),
                        record.file().unwrap_or("unknown"),
                        record.line().unwrap_or(0),
                        message
                    ))
                })
                .build(),
        )
        // Add OS and dialog functionality plugins
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(prelude::ConvState::default()))
        .invoke_handler(spectra_builder.invoke_handler())
        .setup(move |app| {
            spectra_builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
