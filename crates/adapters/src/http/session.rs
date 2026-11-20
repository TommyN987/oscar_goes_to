//! The session operations, and the cookie that carries a session (spec rules 3.2.1 and 3.2.2).

use application::actor::Actor;
use application::session::{self, SignedIn};
use application::session_token::SessionToken;
use axum::Json;
use axum::extract::{FromRequestParts, State};
use axum::http::header::{COOKIE, SET_COOKIE};
use axum::http::request::Parts;
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use domain::player::PlayerName;
use serde::Serialize;
use utoipa::ToSchema;

use super::ApiState;
use super::error::ApiError;

/// The prefix makes the browser itself refuse the cookie unless it is Secure, host-only and for every path.
const SESSION_COOKIE: &str = "__Host-session";

/// Script cannot read the cookie, it travels over HTTPS only, and no cross-site request carries it
/// except a top-level navigation. No `Domain` attribute, so it is host-only.
const COOKIE_ATTRIBUTES: &str = "Path=/; HttpOnly; Secure; SameSite=Lax";

/// The longest a browser keeps a cookie. The server ends a session 90 days after its last use (spec rule 3.2.4);
/// the cookie only has to outlive that.
const COOKIE_MAX_AGE_SECONDS: u32 = 400 * 24 * 60 * 60;

/// One response header, as a handler returns it.
pub type SetCookie = [(HeaderName, HeaderValue); 1];

fn set_cookie(value: &str, max_age_seconds: u32) -> SetCookie {
    let cookie = format!("{SESSION_COOKIE}={value}; Max-Age={max_age_seconds}; {COOKIE_ATTRIBUTES}");
    let cookie =
        HeaderValue::from_str(&cookie).expect("hexadecimal digits and the fixed attributes are a header value");
    [(SET_COOKIE, cookie)]
}

/// Gives the browser the token of the session it now holds.
#[cfg_attr(
    not(feature = "fake-identity"),
    expect(
        dead_code,
        reason = "until the sign-in callback of plan item P3.4, only the fake sign-in issues a session"
    )
)]
pub fn session_cookie(token: &SessionToken) -> SetCookie {
    let in_hex: String = token.expose_bytes().iter().map(|byte| format!("{byte:02x}")).collect();
    set_cookie(&in_hex, COOKIE_MAX_AGE_SECONDS)
}

/// Makes the browser forget its session.
fn cleared_session_cookie() -> SetCookie {
    set_cookie("", 0)
}

/// The session token a request carries, if it carries one this server could have issued.
pub fn presented_token(headers: &HeaderMap) -> Option<SessionToken> {
    headers
        .get_all(COOKIE)
        .iter()
        .filter_map(|header| header.to_str().ok())
        .flat_map(|header| header.split(';'))
        .filter_map(|cookie| cookie.trim().split_once('='))
        .find(|(name, _)| *name == SESSION_COOKIE)
        .and_then(|(_, value)| token_from_hex(value))
}

/// Reads 64 lowercase hexadecimal digits, the only form `session_cookie` writes.
fn token_from_hex(text: &str) -> Option<SessionToken> {
    let (pairs, []) = text.as_bytes().as_chunks::<2>() else {
        return None;
    };
    let mut bytes = [0; 32];
    if pairs.len() != bytes.len() {
        return None;
    }
    for (byte, [high, low]) in bytes.iter_mut().zip(pairs) {
        *byte = (hex_digit(*high)? << 4) | hex_digit(*low)?;
    }
    Some(SessionToken::from_bytes(bytes))
}

fn hex_digit(digit: u8) -> Option<u8> {
    match digit {
        b'0'..=b'9' => Some(digit - b'0'),
        b'a'..=b'f' => Some(digit - b'a' + 10),
        _ => None,
    }
}

/// Who the request acts for, read from its session cookie. A request without a live session is anonymous;
/// each use case decides what that means for it.
pub struct RequestActor(pub Actor);

impl FromRequestParts<ApiState> for RequestActor {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &ApiState) -> Result<Self, Self::Rejection> {
        let presented = presented_token(&parts.headers);
        let actor = session::authenticate(
            presented.as_ref(),
            &state.storage,
            state.clock.as_ref(),
            &state.admin_emails,
        )
        .await?;
        Ok(Self(actor))
    }
}

/// The signed-in player, as only that player sees themself.
#[derive(Serialize, ToSchema)]
pub struct WhoAmI {
    player_id: i64,
    email: String,
    /// Absent until the player has chosen one.
    display_name: Option<String>,
    /// The name to offer while the player has chosen none.
    suggested_name: Option<String>,
    is_admin: bool,
}

impl From<SignedIn<'_>> for WhoAmI {
    fn from(signed_in: SignedIn<'_>) -> Self {
        let (display_name, suggested_name) = match &signed_in.player.name {
            PlayerName::Chosen(name) => (Some(name.as_str().to_owned()), None),
            PlayerName::NotChosen { suggested } => (None, suggested.clone()),
        };
        Self {
            player_id: signed_in.player.id.value(),
            email: signed_in.player.email.clone(),
            display_name,
            suggested_name,
            is_admin: signed_in.is_admin,
        }
    }
}

/// Who am I
///
/// The signed-in player, and whether the configuration names them an admin (spec rule 3.3.4).
#[utoipa::path(
    get,
    path = "/session",
    operation_id = "who_am_i",
    tag = "session",
    responses(
        (status = OK, description = "The signed-in player.", body = WhoAmI),
        (status = UNAUTHORIZED, description = "The request carries no live session.", body = ApiError),
        (status = INTERNAL_SERVER_ERROR, description = "The server failed.", body = ApiError),
    )
)]
pub async fn who_am_i(RequestActor(actor): RequestActor) -> Result<Json<WhoAmI>, ApiError> {
    Ok(Json(session::who_am_i(&actor)?.into()))
}

/// Sign out
///
/// Ends every session of the signed-in player, on every device (spec rule 3.2.5).
#[utoipa::path(
    post,
    path = "/session/sign-out",
    operation_id = "sign_out",
    tag = "session",
    responses(
        (status = NO_CONTENT, description = "The player is signed out everywhere."),
        (status = UNAUTHORIZED, description = "The request carries no live session.", body = ApiError),
        (status = INTERNAL_SERVER_ERROR, description = "The server failed.", body = ApiError),
    )
)]
pub async fn sign_out(
    State(state): State<ApiState>,
    RequestActor(actor): RequestActor,
) -> Result<(StatusCode, SetCookie), ApiError> {
    session::sign_out(&actor, &state.storage).await?;
    Ok((StatusCode::NO_CONTENT, cleared_session_cookie()))
}
