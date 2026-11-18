//! The one JSON shape every API error shares (spec rule 6.4.1).

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// A stable, machine-readable error code. Clients branch on it, so a code is never renamed.
#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    NotFound,
}

impl ErrorCode {
    fn status(self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

#[derive(Serialize)]
pub struct ApiError {
    code: ErrorCode,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.code.status(), Json(self)).into_response()
    }
}

/// Answers any `/api` path that is not an operation (spec rule 2.4.2).
pub async fn unknown_operation() -> ApiError {
    ApiError {
        code: ErrorCode::NotFound,
        message: "No such API operation.".into(),
    }
}
