use thiserror::Error;
use wasm_bindgen::JsValue;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Window not found")]
    WindowNotFound,

    #[error("Document not found")]
    DocumentNotFound,

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("Serialization error: {0}")]
    SerializationError(serde_json::Error),

    #[error("DOM operation failed: {0}")]
    DomOperationError(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Cryptography error: {0}")]
    CryptoError(String),
}

impl From<AppError> for JsValue {
    fn from(error: AppError) -> Self {
        JsValue::from_str(&error.to_string())
    }
}

impl From<JsValue> for AppError {
    fn from(value: JsValue) -> Self {
        AppError::WebSocketError(value.as_string().unwrap_or_default())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        AppError::SerializationError(error)
    }
}

impl From<base64::DecodeError> for AppError {
    fn from(error: base64::DecodeError) -> Self {
        AppError::CryptoError(format!("Base64 cannot be decoded: {}", error))
    }
}
