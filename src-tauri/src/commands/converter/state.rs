//! State management module for conversion operations
//!
//! This module handles getting, setting, and resetting conversion state.

use crate::prelude::*;
use log::{debug, info, trace};
use tauri::State;
use tokio::sync::Mutex;

/// Updates a specific field in the conversion state.
///
/// # Arguments
/// * `input` - Key-value pair specifying which state field to update and its new value
/// * `state` - Application state containing conversion parameters
///
/// # Returns
/// * `EResult<BaseResponse>` - Success response with execution duration
#[tauri::command(async)]
#[specta::specta]
pub async fn conv_state_set(
    input: ConvStateKey,
    state: State<'_, Mutex<ConvState>>,
) -> EResult<BaseResponse> {
    info!(
        "Setting conversion state key: {}",
        match &input {
            ConvStateKey::Name(_) => "Name",
            ConvStateKey::Source(_) => "Source",
            ConvStateKey::Target(_) => "Target",
            ConvStateKey::BundleFlag(_) => "BundleFlag",
            ConvStateKey::Direction(_) => "Direction",
            ConvStateKey::Format(_) => "Format",
            ConvStateKey::CreateDirectory(_) => "CreateDirectory",
            ConvStateKey::ConvertToWebp(_) => "ConvertToWebp",
            ConvStateKey::ImageFormat(_) => "ImageFormat",
            ConvStateKey::VolumeSizes(_) => "VolumeSizes",
            ConvStateKey::Data(_) => "Data",
            ConvStateKey::EditedData(_) => "EditedData",
            ConvStateKey::HideSingleVolumeNumber(_) => "HideSingleVolumeNumber",
            ConvStateKey::VolumeSeparator(_) => "VolumeSeparator",
        }
    );
    let start = std::time::Instant::now();

    let mut state = state.lock().await;

    // Set the state based on the input
    match input {
        ConvStateKey::Name(value) => {
            debug!("Setting name to: {}", value);
            state.name = value;
        }
        ConvStateKey::Source(value) => {
            debug!("Setting source path to: {:?}", value);
            state.source = value;
        }
        ConvStateKey::Target(value) => {
            debug!("Setting target path to: {:?}", value);
            state.target = value;
        }
        ConvStateKey::BundleFlag(value) => {
            debug!("Setting bundle flag to: {:?}", value);
            state.bundle_flag = value;
        }
        ConvStateKey::Direction(value) => {
            debug!("Setting reading direction to: {:?}", value);
            state.direction = value;
        }
        ConvStateKey::Format(value) => {
            debug!("Setting output format to: {:?}", value);
            state.format = value;
        }
        ConvStateKey::CreateDirectory(value) => {
            debug!("Setting create directory flag to: {}", value);
            state.create_directory = value;
        }
        ConvStateKey::ConvertToWebp(value) => {
            debug!("Setting convert to WebP flag to: {}", value);
            state.convert_to_webp = value;
            // Also update image_format for backward compatibility
            state.image_format = if value {
                ImageOutputFormat::WebP
            } else {
                ImageOutputFormat::None
            };
        }
        ConvStateKey::ImageFormat(value) => {
            debug!("Setting image format to: {:?}", value);
            state.image_format = value;
            // Also update convert_to_webp for backward compatibility
            state.convert_to_webp = matches!(value, ImageOutputFormat::WebP);
        }
        ConvStateKey::VolumeSizes(value) => {
            debug!("Setting volume sizes: {:?}", value);
            state.volume_sizes = value;
        }
        ConvStateKey::Data(value) => {
            debug!("Setting data with {} chapters", value.len());
            state.data = value;
        }
        ConvStateKey::EditedData(value) => {
            debug!(
                "Setting edited data: {} chapters",
                value.as_ref().map_or(0, |v| v.len())
            );
            state.edited_data = value;
        }
        ConvStateKey::HideSingleVolumeNumber(value) => {
            debug!("Setting hide single volume number flag to: {}", value);
            state.hide_single_volume_number = value;
        }
        ConvStateKey::VolumeSeparator(value) => {
            debug!("Setting volume separator to: {}", value);
            state.volume_separator = value;
        }
    }

    let duration = start.elapsed().as_secs_f64();
    trace!("State updated in {}s", duration);
    Ok(BaseResponse::default_duration(duration))
}

/// Retrieves the complete current conversion state.
///
/// # Arguments
/// * `state` - Application state containing conversion parameters
///
/// # Returns
/// * `EResult<BaseResponse<ConvState>>` - Success response containing the current state
#[tauri::command(async)]
#[specta::specta]
pub async fn conv_state_get(
    state: State<'_, Mutex<ConvState>>,
) -> EResult<BaseResponse<ConvState>> {
    info!("Getting conversion state");
    let start = std::time::Instant::now();

    let state = state.lock().await;
    trace!("Acquired state lock");

    let duration = start.elapsed().as_secs_f64();
    debug!("Retrieved conversion state in {}s", duration);
    Ok(BaseResponse {
        duration,
        comment: None,
        payload: Some(state.clone()),
    })
}

/// Resets the conversion state to default values.
///
/// # Arguments
/// * `state` - Application state containing conversion parameters
///
/// # Returns
/// * `EResult<BaseResponse>` - Success response with execution duration
#[tauri::command(async)]
#[specta::specta]
pub async fn conv_state_reset(state: State<'_, Mutex<ConvState>>) -> EResult<BaseResponse> {
    info!("Resetting conversion state");
    let start = std::time::Instant::now();

    let mut state = state.lock().await;
    trace!("Acquired state lock");
    state.reset();
    debug!("State reset to defaults");

    let duration = start.elapsed().as_secs_f64();
    debug!("Reset completed in {}s", duration);
    Ok(BaseResponse::default_duration(duration))
}
