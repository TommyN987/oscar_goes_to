//! Refuses a request whose body is larger than any operation needs (spec rule 6.5.2).

use axum::extract::Request;
use axum::http::header::CONTENT_LENGTH;
use axum::middleware::Next;
use axum::response::Response;

use super::MAX_REQUEST_BODY_BYTES;
use super::error::{ApiError, ErrorCode};

/// Answers 413 when the declared `Content-Length` exceeds the limit, before any of the body is read.
/// A body with no declared length passes here; plan item P3.3 caps that one where a body is read.
pub async fn refuse_oversize_body(request: Request, next: Next) -> Result<Response, ApiError> {
    let declared_length = request
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|length| length.to_str().ok()?.parse::<usize>().ok());
    if declared_length.is_some_and(|length| length > MAX_REQUEST_BODY_BYTES) {
        return Err(ApiError::new(
            ErrorCode::PayloadTooLarge,
            "The request body is too large.",
        ));
    }
    Ok(next.run(request).await)
}
