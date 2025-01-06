use std::io;

use serde::Serialize;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error, Serialize)]
pub enum AppError {
    #[error("tauri error: {0}")]
    TauriError(String),
    #[error("tauri global shortcut error: {0}")]
    TauriShortcutError(#[from] tauri_plugin_global_shortcut::Error),
    #[error("io error: {0}")]
    IOError(String),
    #[error("other error {0}")]
    OtherError(String),
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        error!("aynhow error: {}", err.to_string());
        AppError::OtherError(err.to_string())
    }
}

impl From<tauri::Error> for AppError {
    fn from(err: tauri::Error) -> Self {
        error!("tauri error: {}", err.to_string());
        AppError::TauriError(err.to_string())
    }
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        error!("io error: {}", err.to_string());
        AppError::IOError(err.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
