//! The fake identity adapter: sign-in as a player of the caller's choice, without Google (spec rule 2.2.3).
//!
//! This module is compiled only with the `fake-identity` feature, which the release image is built without
//! (spec rule 3.4.1). It is no API operation, so `openapi.json` never lists it (spec rule 4.6.1).

use application::session;
use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use domain::player::{IdentityClaims, VerifiedIdentity};
use serde::Deserialize;

use super::ApiState;
use super::error::ApiError;
use super::session::{SetCookie, presented_token, session_cookie};

/// Where the route sits under `/api`. `scripts/smoke-image.sh` probes the same path on the release image.
pub const PATH: &str = "/fake-identity/sign-in";

/// The account the caller wants to be. The fake provider asserts its email as verified.
#[derive(Deserialize)]
pub struct ChosenIdentity {
    subject: String,
    email: String,
    name: Option<String>,
}

#[expect(
    clippy::expect_used,
    reason = "the claims built here say the email is verified; plan item P3.4 maps the refusal to an API error instead"
)]
pub async fn sign_in(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(chosen): Json<ChosenIdentity>,
) -> Result<(StatusCode, SetCookie), ApiError> {
    let identity = VerifiedIdentity::from_claims(IdentityClaims {
        subject: chosen.subject,
        email: chosen.email,
        email_verified: true,
        name: chosen.name,
    })
    .expect("the claims say the email is verified");

    let token = session::sign_in(
        &identity,
        presented_token(&headers).as_ref(),
        &state.storage,
        state.clock.as_ref(),
    )
    .await?;
    Ok((StatusCode::NO_CONTENT, session_cookie(&token)))
}
