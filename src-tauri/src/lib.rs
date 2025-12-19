//! Main application initialization module.

mod commands;
mod events;
mod prelude;
mod setup;
mod state;

#[cfg(debug_assertions)]
use specta_typescript::Typescript;
use tauri::{Builder, Manager};
use tauri_specta::{collect_commands, collect_events, Builder as SpectaBuilder};
use tokio::sync::Mutex;

// Use mimalloc as the global allocator for better multi-threaded performance
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Register commands and events
    let spectra_builder = SpectaBuilder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::state::conv_state_set,
            commands::state::conv_state_get,
            commands::state::conv_state_reset,
            // ---
            commands::analyze::conv_analyze,
            commands::bundle::conv_bundle,
            commands::convert::conv_convert,
        ])
        .events(collect_events![
            events::VolumeStartEvent,
            events::VolumeCompleteEvent,
            events::ImageProgressEvent,
            events::ConversionStartEvent,
            events::ConversionCompleteEvent,
            events::StatusMessageEvent,
        ]);

    #[cfg(debug_assertions)]
    {
        let mut ts = Typescript::default();
        ts = ts.bigint(specta_typescript::BigIntExportBehavior::Number);

        spectra_builder
            .export(ts, "../src/types/bindings.ts")
            .expect("Failed to export typescript bindings");
    }

    Builder::default()
        // Single Instance
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        // Logging (from setup module)
        .plugin(setup::logging::init().build())
        // Plugins
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        // State (from state module)
        .manage(Mutex::new(state::ConvState::default()))
        // Invoke Handler
        .invoke_handler(spectra_builder.invoke_handler())
        .setup(move |app| {
            // Initialize performance (from setup module)
            setup::concurrency::init();

            spectra_builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
