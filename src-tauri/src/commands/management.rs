use crate::db::sync::SyncManager;
use crate::prelude::*;
use std::path::PathBuf;
use tauri::{Manager, State};

/// Starts the database synchronization service.
///
/// # Arguments
/// * `interval` - Time interval in minutes between syncs
/// * `app_handle` - Tauri application handle
/// * `sync_manager` - State-managed sync manager instance
///
/// # Returns
/// * `EResult<BaseResponse>` - Success response or error
#[tauri::command(async)]
#[specta::specta]
pub async fn mgmt_sync_start(
    interval: Option<u64>,
    app_handle: tauri::AppHandle,
    sync_manager: State<'_, SyncManager>,
) -> EResult<BaseResponse> {
    let start = std::time::Instant::now();

    // Stop any existing sync process first
    sync_manager.stop();

    // Create the new manager with an updated interval or continue with the existing one
    if let Some(new_interval) = interval {
        let new_manager = SyncManager::new(new_interval);
        new_manager.start();

        // Update the managed state
        app_handle.manage(new_manager);
    } else {
        // Restart with existing configuration
        sync_manager.start();
    }

    Ok(BaseResponse::default_duration(
        start.elapsed().as_secs_f64(),
    ))
}

/// Stops the database synchronization service.
///
/// # Arguments
/// * `sync_manager` - State-managed sync manager instance
///
/// # Returns
/// * `EResult<BaseResponse>` - Success response or error
#[tauri::command(async)]
#[specta::specta]
pub async fn mgmt_sync_stop(sync_manager: State<'_, SyncManager>) -> EResult<BaseResponse> {
    let start = std::time::Instant::now();

    sync_manager.stop();

    Ok(BaseResponse::default_duration(
        start.elapsed().as_secs_f64(),
    ))
}

/// Retrieves the current status of the synchronization service.
///
/// # Arguments
/// * `sync_manager` - State-managed sync manager instance
///
/// # Returns
/// * `EResult<BaseResponse<SyncStatus>>` - Status containing next sync time information
#[tauri::command(async)]
#[specta::specta]
pub async fn mgmt_sync_status(
    sync_manager: State<'_, SyncManager>,
) -> EResult<BaseResponse<SyncStatus>> {
    let start = std::time::Instant::now();

    let time_until_next = sync_manager.time_until_next_sync();

    let status = SyncStatus {
        seconds_until_next_sync: time_until_next.map(|d| d.as_secs()),
        minutes_until_next_sync: time_until_next.map(|d| d.as_secs() / 60),
    };

    Ok(BaseResponse {
        duration: start.elapsed().as_secs_f64(),
        comment: None,
        payload: Some(status),
    })
}

/// Retrieves the path to the application log files.
///
/// # Arguments
/// * `app_handle` - Tauri application handle
///
/// # Returns
/// * `EResult<BaseResponse<LogPath>>` - Path information for log files
#[tauri::command(async)]
#[specta::specta]
pub async fn mgmt_get_logs_path(app_handle: tauri::AppHandle) -> EResult<BaseResponse<LogPath>> {
    let start = std::time::Instant::now();

    // Get the path to app log directory
    let log_dir = app_handle
        .path()
        .app_log_dir()
        .unwrap_or_else(|_| PathBuf::from("."));

    // The log file is named "logs" as configured in lib.rs
    let log_file = log_dir.join("logs");

    Ok(BaseResponse {
        duration: start.elapsed().as_secs_f64(),
        comment: None,
        payload: Some(LogPath {
            directory: log_dir.to_string_lossy().to_string(),
            file: log_file.to_string_lossy().to_string(),
        }),
    })
}
