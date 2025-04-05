use crate::db::sync::SyncManager;
use crate::prelude::*;
use log::{debug, error, info, trace};
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
    info!(
        "Starting database sync service, interval: {:?} minutes",
        interval
    );
    let start = std::time::Instant::now();

    // Stop any existing sync process first
    debug!("Stopping existing sync process");
    sync_manager.stop();

    // Create the new manager with an updated interval or continue with the existing one
    if let Some(new_interval) = interval {
        debug!(
            "Creating new sync manager with interval {} minutes",
            new_interval
        );
        let new_manager = SyncManager::new(new_interval);
        new_manager.start();

        // Update the managed state
        trace!("Updating managed sync state with new interval");
        app_handle.manage(new_manager);
    } else {
        debug!("Restarting sync with existing configuration");
        sync_manager.start();
    }

    let duration = start.elapsed().as_secs_f64();
    debug!("Database sync service started, duration: {}s", duration);
    Ok(BaseResponse::default_duration(duration))
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
    info!("Stopping database sync service");
    let start = std::time::Instant::now();

    sync_manager.stop();

    let duration = start.elapsed().as_secs_f64();
    debug!("Database sync service stopped, duration: {}s", duration);
    Ok(BaseResponse::default_duration(duration))
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
    info!("Getting sync service status");
    let start = std::time::Instant::now();

    let time_until_next = sync_manager.time_until_next_sync();
    debug!("Time until next sync: {:?}", time_until_next);

    let status = SyncStatus {
        seconds_until_next_sync: time_until_next.map(|d| d.as_secs()),
        minutes_until_next_sync: time_until_next.map(|d| d.as_secs() / 60),
    };

    trace!(
        "Sync status: seconds={:?}, minutes={:?}",
        status.seconds_until_next_sync,
        status.minutes_until_next_sync
    );

    let duration = start.elapsed().as_secs_f64();
    debug!("Retrieved sync status, duration: {}s", duration);
    Ok(BaseResponse {
        duration,
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
    info!("Getting application logs path");
    let start = std::time::Instant::now();

    // Get the path to app log directory
    debug!("Retrieving app log directory path");
    let log_dir = match app_handle.path().app_log_dir() {
        Ok(dir) => {
            trace!("App log directory: {:?}", dir);
            dir
        }
        Err(e) => {
            error!("Failed to get app log directory: {}", e);
            PathBuf::from(".")
        }
    };

    // The log file is named "logs" as configured in lib.rs
    let log_file = log_dir.join("logs");
    debug!("Log file path: {:?}", log_file);

    let duration = start.elapsed().as_secs_f64();
    debug!("Retrieved logs path, duration: {}s", duration);
    Ok(BaseResponse {
        duration,
        comment: None,
        payload: Some(LogPath {
            directory: log_dir.to_string_lossy().to_string(),
            file: log_file.to_string_lossy().to_string(),
        }),
    })
}
