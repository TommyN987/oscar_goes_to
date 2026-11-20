//! The one JSON shape every API error shares (spec rule 6.4.1).

use std::error::Error;

use application::session::{NotSignedIn, SignOutError};
use application::storage::StorageError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use utoipa::ToSchema;

/// A stable, machine-readable error code. Clients branch on it, so a code is never renamed.
#[derive(Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    NotFound,
    PayloadTooLarge,
    DatabaseUnavailable,
    NotSignedIn,
    Internal,
}

impl ErrorCode {
    fn status(self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::DatabaseUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::NotSignedIn => StatusCode::UNAUTHORIZED,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ApiError {
    code: ErrorCode,
    message: String,
}

impl ApiError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.code.status(), Json(self)).into_response()
    }
}

impl From<NotSignedIn> for ApiError {
    fn from(_: NotSignedIn) -> Self {
        Self::new(ErrorCode::NotSignedIn, "Sign in to continue.")
    }
}

/// The cause goes to the log, in the error's message; the client learns only that the failure is not its own.
impl From<StorageError> for ApiError {
    fn from(error: StorageError) -> Self {
        tracing::error!(error = &error as &dyn Error, "storage failed");
        Self::new(ErrorCode::Internal, "Something went wrong. Try again.")
    }
}

impl From<SignOutError> for ApiError {
    fn from(error: SignOutError) -> Self {
        match error {
            SignOutError::NotSignedIn(refusal) => refusal.into(),
            SignOutError::Storage(failure) => failure.into(),
        }
    }
}

/// Answers any `/api` path that is not an operation (spec rule 2.4.2).
pub async fn unknown_operation() -> ApiError {
    ApiError::new(ErrorCode::NotFound, "No such API operation.")
}
