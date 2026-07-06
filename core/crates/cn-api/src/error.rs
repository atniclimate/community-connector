use serde::Serialize;
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ErrorCode {
    NotFound,
    InvalidJson,
    InvalidViewer,
    UnsupportedSchemaVersion,
    GroupExists,
    GroupNotLoaded,
    LoadNotBegun,
    #[allow(dead_code)]
    Denied,
    Internal,
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub(crate) struct ApiError {
    pub(crate) code: ErrorCode,
    pub(crate) message: String,
    pub(crate) details: Value,
}

impl ApiError {
    pub(crate) fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: json!({}),
        }
    }

    pub(crate) fn with_details(
        code: ErrorCode,
        message: impl Into<String>,
        details: Value,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            details,
        }
    }

    pub(crate) fn not_found() -> Self {
        Self::new(ErrorCode::NotFound, "not found")
    }

    pub(crate) fn invalid_json(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidJson, message)
    }

    pub(crate) fn invalid_viewer(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidViewer, message)
    }

    pub(crate) fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, message)
    }
}

#[derive(Serialize)]
pub(crate) struct ErrorEnvelope {
    pub(crate) code: ErrorCode,
    pub(crate) message: String,
    pub(crate) details: Value,
}

impl From<ApiError> for ErrorEnvelope {
    fn from(value: ApiError) -> Self {
        Self {
            code: value.code,
            message: value.message,
            details: value.details,
        }
    }
}
