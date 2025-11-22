use crate::prelude::*;
use log::{debug, error, info, trace};
use std::path::PathBuf;
use tauri::Manager;

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
