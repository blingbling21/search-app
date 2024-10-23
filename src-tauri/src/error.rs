use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
pub enum AppError {
    #[error("tauri error: {0}")]
    TauriError(String),
    #[error("tauri global shortcut error: {0}")]
    TauriShortcutError(#[from] tauri_plugin_global_shortcut::Error),
    #[error("other error {0}")]
    OtherError(String),
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::OtherError(err.to_string())
    }
}

impl From<tauri::Error> for AppError {
    fn from(err: tauri::Error) -> Self {
        AppError::TauriError(err.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
