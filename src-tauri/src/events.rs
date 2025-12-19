use crate::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::AppHandle;
use tauri_specta::Event;

/// Emitted when a volume conversion starts
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct VolumeStartEvent {
    pub volume_index: usize,
    pub total_volumes: usize,
    pub volume_name: String,
}

/// Emitted when a volume conversion completes
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct VolumeCompleteEvent {
    pub volume_index: usize,
    pub total_volumes: usize,
    pub volume_name: String,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Emitted periodically to show image processing progress
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct ImageProgressEvent {
    pub volume_index: usize,
    pub volume_name: String,
    pub current_image: usize,
    pub total_images: usize,
}

/// Emitted when conversion batch starts
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct ConversionStartEvent {
    pub total_volumes: usize,
}

/// Emitted when all conversions complete
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct ConversionCompleteEvent {
    pub total_volumes: usize,
    pub successful: usize,
    pub failed: usize,
    pub duration_seconds: f64,
}

/// Status message types for live feedback
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StatusMessageType {
    VolumeStarted {
        volume_index: usize,
        volume_name: String,
    },
    PageAdded {
        volume_index: usize,
        volume_name: String,
        page_number: usize,
        total_pages: usize,
    },
    VolumeFinished {
        volume_index: usize,
        volume_name: String,
        success: bool,
    },
}

/// Emitted for live status updates during conversion
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct StatusMessageEvent {
    pub message: StatusMessageType,
    pub timestamp: u64,
}

// --- Helper Functions ---

pub fn emit_volume_start(
    app: &AppHandle,
    volume_index: usize,
    total_volumes: usize,
    volume_name: String,
) -> EResult<()> {
    VolumeStartEvent {
        volume_index,
        total_volumes,
        volume_name,
    }
    .emit(app)
    .map_err(|e| Error::AsyncTaskError(format!("Failed to emit volume start event: {}", e)))
}

pub fn emit_volume_complete(
    app: &AppHandle,
    volume_index: usize,
    total_volumes: usize,
    volume_name: String,
    success: bool,
    error_message: Option<String>,
) -> EResult<()> {
    VolumeCompleteEvent {
        volume_index,
        total_volumes,
        volume_name,
        success,
        error_message,
    }
    .emit(app)
    .map_err(|e| Error::AsyncTaskError(format!("Failed to emit volume complete event: {}", e)))
}

pub fn emit_image_progress(
    app: &AppHandle,
    volume_index: usize,
    volume_name: String,
    current_image: usize,
    total_images: usize,
) -> EResult<()> {
    ImageProgressEvent {
        volume_index,
        volume_name,
        current_image,
        total_images,
    }
    .emit(app)
    .map_err(|e| Error::AsyncTaskError(format!("Failed to emit image progress event: {}", e)))
}

pub fn emit_conversion_start(app: &AppHandle, total_volumes: usize) -> EResult<()> {
    ConversionStartEvent { total_volumes }
        .emit(app)
        .map_err(|e| Error::AsyncTaskError(format!("Failed to emit conversion start event: {}", e)))
}

pub fn emit_conversion_complete(
    app: &AppHandle,
    total_volumes: usize,
    successful: usize,
    failed: usize,
    duration_seconds: f64,
) -> EResult<()> {
    ConversionCompleteEvent {
        total_volumes,
        successful,
        failed,
        duration_seconds,
    }
    .emit(app)
    .map_err(|e| Error::AsyncTaskError(format!("Failed to emit conversion complete event: {}", e)))
}
