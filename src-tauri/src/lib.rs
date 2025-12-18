//! Main application initialization module.
//!
//! This module is responsible for setting up and running the Tauri application.
//! It handles command registration, TypeScript bindings export, plugin setup,
//! and application state initialization.

use crate::commands::converter::{
    conv_analyze, conv_bundle, conv_convert, conv_state_get, conv_state_reset, conv_state_set,
    ConversionCompleteEvent, ConversionStartEvent, ImageProgressEvent, StatusMessageEvent,
    VolumeCompleteEvent, VolumeStartEvent,
};
use crate::commands::management::mgmt_get_logs_path;
#[cfg(debug_assertions)]
use specta_typescript::Typescript;
use tauri::{Builder, Manager};
use tauri_specta::{collect_commands, collect_events, Builder as SpectaBuilder};
use tokio::sync::Mutex;

mod commands;
mod prelude;

// Use mimalloc as the global allocator for better multi-threaded performance -> Leads to speedups but larger binary size
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Initialize the rayon global thread pool using ResourceBudget settings
///
/// This configures rayon for optimal performance based on system resources
fn init_rayon_thread_pool() {
    use common::ResourceBudget;

    let budget = ResourceBudget::calculate();

    // Set stack size to 4MB per thread (image processing can be stack-heavy)
    let stack_size = 4 * 1024 * 1024;

    match rayon::ThreadPoolBuilder::new()
        .num_threads(budget.thread_pool_size)
        .stack_size(stack_size)
        .thread_name(|idx| format!("rayon-worker-{}", idx))
        .build_global()
    {
        Ok(_) => {
            log::info!(
                "Initialized rayon thread pool with {} threads (stack size: {}MB)",
                budget.thread_pool_size,
                stack_size / (1024 * 1024)
            );
        }
        Err(_) => {
            // Thread pool already initialized, which is fine
            log::info!("Rayon thread pool already initialized, using default configuration");
        }
    }
}

/// Initializes and runs the Tauri application.
///
/// This function performs the following tasks:
/// 1. Initialize optimized rayon thread pool
/// 2. Register commands for frontend-backend communication
/// 3. Export TypeScript bindings for development build
/// 4. Configure the application with necessary plugins
/// 5. Set up application state, including database initialization
/// 6. Start the Tauri application with the configured settings
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize rayon thread pool early for optimal performance
    init_rayon_thread_pool();

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
        // Configure application logging with different levels for debug/release
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: None,
                    }),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                ])
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
                .level(if cfg!(debug_assertions) {
                    log::LevelFilter::Debug
                } else {
                    // Warn
                    log::LevelFilter::Debug
                })
                .filter(|metadata| metadata.target().starts_with(env!("CARGO_PKG_NAME")))
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
