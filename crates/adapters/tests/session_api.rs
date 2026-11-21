//! The session operations through the router, signed in through the fake identity route.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a test fails by panicking, in its helper functions too"
)]
#![cfg(feature = "fake-identity")]

use std::sync::Arc;

use adapters::clock::SystemClock;
use adapters::http::{ApiState, openapi, router};
use adapters::telemetry::Telemetry;
use application::admin_emails::AdminEmails;
use axum::Router;
use axum::body::Body;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE, COOKIE, SET_COOKIE};
use axum::http::{Request, Response, StatusCode};
use http_body_util::BodyExt;
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;

const FAKE_SIGN_IN: &str = "/api/fake-identity/sign-in";

fn app_with_admins(database: PgPool, admin_emails: AdminEmails) -> Router {
    let state = ApiState::new(database, Arc::new(SystemClock), admin_emails);
    router(None, state, Telemetry::disabled())
}

fn app(database: PgPool) -> Router {
    app_with_admins(database, AdminEmails::none())
}

async fn send(app: &Router, request: Request<Body>) -> Response<Body> {
    app.clone().oneshot(request).await.unwrap()
}

/// Signs in as Pat through the fake identity route, presenting `cookie` if there is one.
async fn sign_in_as_pat(app: &Router, cookie: Option<&str>) -> Response<Body> {
    let mut request = Request::post(FAKE_SIGN_IN).header(CONTENT_TYPE, "application/json");
    if let Some(cookie) = cookie {
        request = request.header(COOKIE, cookie);
    }
    let body = json!({ "subject": "subject-pat", "email": "pat@example.com", "name": "Pat Player" });
    send(app, request.body(Body::from(body.to_string())).unwrap()).await
}

fn set_cookie(response: &Response<Body>) -> &str {
    response
        .headers()
        .get(SET_COOKIE)
        .expect("the response sets a cookie")
        .to_str()
        .unwrap()
}

/// The `name=value` a browser would send back after `response`.
fn cookie_of(response: &Response<Body>) -> String {
    set_cookie(response).split(';').next().unwrap().to_owned()
}

async fn signed_in_cookie(app: &Router) -> String {
    cookie_of(&sign_in_as_pat(app, None).await)
}

async fn who_am_i(app: &Router, cookie: Option<&str>) -> Response<Body> {
    let mut request = Request::get("/api/session");
    if let Some(cookie) = cookie {
        request = request.header(COOKIE, cookie);
    }
    send(app, request.body(Body::empty()).unwrap()).await
}

async fn sign_out(app: &Router, cookie: Option<&str>) -> Response<Body> {
    let mut request = Request::post("/api/session/sign-out");
    if let Some(cookie) = cookie {
        request = request.header(COOKIE, cookie);
    }
    send(app, request.body(Body::empty()).unwrap()).await
}

async fn body_json(response: Response<Body>) -> serde_json::Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[sqlx::test]
async fn rule_2_2_3_a_fake_player_of_the_callers_choice_signs_in(database: PgPool) {
    let app = app(database);

    let signed_in = sign_in_as_pat(&app, None).await;
    assert_eq!(signed_in.status(), StatusCode::NO_CONTENT);

    let answer = who_am_i(&app, Some(&cookie_of(&signed_in))).await;
    assert_eq!(answer.status(), StatusCode::OK);
    let mut who = body_json(answer).await;
    assert!(who["player_id"].take().is_i64());
    assert_eq!(
        who,
        json!({
            "player_id": null,
            "email": "pat@example.com",
            "display_name": null,
            "suggested_name": "Pat Player",
            "is_admin": false,
        })
    );
}

/// Asserts what spec rule 3.2.2 asks of a `Set-Cookie` header for the session cookie.
/// A browser refuses a `__Host-` cookie that lacks any of Secure, `Path=/` and the absence of Domain,
/// so the header that clears the cookie must meet the rule as the one that sets it does.
fn assert_http_only_secure_lax_and_host_only(set_cookie: &str) {
    let attributes: Vec<&str> = set_cookie.split("; ").skip(1).collect();
    assert!(set_cookie.starts_with("__Host-session="));
    assert!(attributes.contains(&"HttpOnly"));
    assert!(attributes.contains(&"Secure"));
    assert!(attributes.contains(&"SameSite=Lax"));
    // Host-only: the browser sends it to this host alone, because no Domain attribute widens it.
    assert!(attributes.contains(&"Path=/"));
    assert!(!attributes.iter().any(|attribute| attribute.starts_with("Domain")));
}

#[sqlx::test]
async fn rule_3_2_2_the_session_cookie_is_http_only_secure_lax_and_host_only(database: PgPool) {
    let signed_in = sign_in_as_pat(&app(database), None).await;

    assert_http_only_secure_lax_and_host_only(set_cookie(&signed_in));
}

#[sqlx::test]
async fn rule_3_2_2_the_cookie_that_signs_out_has_the_same_attributes(database: PgPool) {
    let app = app(database);
    let cookie = signed_in_cookie(&app).await;

    let signed_out = sign_out(&app, Some(&cookie)).await;

    assert_http_only_secure_lax_and_host_only(set_cookie(&signed_out));
}

#[sqlx::test]
async fn rule_3_2_1_the_cookie_value_appears_nowhere_in_the_database(database: PgPool) {
    let cookie = signed_in_cookie(&app(database.clone())).await;
    let token = cookie.strip_prefix("__Host-session=").unwrap();

    // Every row as JSON text, in which Postgres writes a byte column in hexadecimal, as the cookie is.
    let rows: Vec<String> = sqlx::query_scalar(
        "SELECT row_to_json(players)::text FROM players
         UNION ALL SELECT row_to_json(sessions)::text FROM sessions
         UNION ALL SELECT row_to_json(blocked_accounts)::text FROM blocked_accounts",
    )
    .fetch_all(&database)
    .await
    .unwrap();

    assert_eq!(token.len(), 64, "256 bits in hexadecimal");
    assert_eq!(rows.len(), 2, "one player and one session");
    assert!(rows.iter().all(|row| !row.contains(token)));
}

#[sqlx::test]
async fn who_am_i_without_a_live_session_is_401(database: PgPool) {
    let app = app(database);

    let without_a_cookie = who_am_i(&app, None).await;
    let with_a_cookie_never_issued = who_am_i(&app, Some(&format!("__Host-session={}", "ab".repeat(32)))).await;
    let with_a_malformed_cookie = who_am_i(&app, Some("__Host-session=not-a-token")).await;

    assert_eq!(with_a_cookie_never_issued.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(with_a_malformed_cookie.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(without_a_cookie.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(body_json(without_a_cookie).await["code"], "not_signed_in");
}

/// A browser sends a cookie's value as it was set, and another cookie of this host may hold any bytes.
#[sqlx::test]
async fn the_session_cookie_is_found_beside_a_cookie_that_is_not_ascii(database: PgPool) {
    let app = app(database);
    let cookie = signed_in_cookie(&app).await;

    let answer = who_am_i(&app, Some(&format!("theme=Grün; {cookie}"))).await;

    assert_eq!(answer.status(), StatusCode::OK);
}

#[sqlx::test]
async fn a_malformed_session_cookie_does_not_hide_a_valid_one(database: PgPool) {
    let app = app(database);
    let cookie = signed_in_cookie(&app).await;

    let answer = who_am_i(&app, Some(&format!("__Host-session=not-a-token; {cookie}"))).await;

    assert_eq!(answer.status(), StatusCode::OK);
}

/// An HTTP/2 client may send each cookie in a header of its own.
#[sqlx::test]
async fn the_session_cookie_is_found_in_a_second_cookie_header(database: PgPool) {
    let app = app(database);
    let cookie = signed_in_cookie(&app).await;

    let request = Request::get("/api/session")
        .header(COOKIE, "theme=dark")
        .header(COOKIE, &cookie)
        .body(Body::empty())
        .unwrap();

    assert_eq!(send(&app, request).await.status(), StatusCode::OK);
}

#[sqlx::test]
async fn a_cookie_whose_name_only_resembles_the_session_cookies_is_ignored(database: PgPool) {
    let app = app(database);
    let cookie = signed_in_cookie(&app).await;
    let token = cookie.strip_prefix("__Host-session=").unwrap();

    let with_a_longer_name = who_am_i(&app, Some(&format!("__Host-session-old={token}"))).await;
    let with_a_prefixed_name = who_am_i(&app, Some(&format!("x__Host-session={token}"))).await;

    assert_eq!(with_a_longer_name.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(with_a_prefixed_name.status(), StatusCode::UNAUTHORIZED);
}

/// An answer that depends on the cookie, or sets one, must not be kept by any cache between server and browser.
#[sqlx::test]
async fn no_cache_may_store_an_api_response(database: PgPool) {
    let app = app(database);
    let signed_in = sign_in_as_pat(&app, None).await;
    let cookie = cookie_of(&signed_in);

    let who = who_am_i(&app, Some(&cookie)).await;
    let refused = who_am_i(&app, None).await;

    for response in [&signed_in, &who, &refused] {
        assert_eq!(response.headers().get(CACHE_CONTROL).unwrap(), "no-store");
    }
}

#[sqlx::test]
async fn the_session_cookie_is_found_among_other_cookies(database: PgPool) {
    let app = app(database);
    let cookie = signed_in_cookie(&app).await;

    let answer = who_am_i(&app, Some(&format!("theme=dark; {cookie}; other=1"))).await;

    assert_eq!(answer.status(), StatusCode::OK);
}

/// Sign-out's part of spec rule 3.2.5; plan item P3.8 closes the rule.
#[sqlx::test]
async fn rule_3_2_5_after_sign_out_the_old_cookie_is_refused(database: PgPool) {
    let app = app(database);
    let cookie = signed_in_cookie(&app).await;

    let signed_out = sign_out(&app, Some(&cookie)).await;

    assert_eq!(signed_out.status(), StatusCode::NO_CONTENT);
    assert!(set_cookie(&signed_out).starts_with("__Host-session=;"));
    assert!(set_cookie(&signed_out).contains("Max-Age=0"));
    assert_eq!(who_am_i(&app, Some(&cookie)).await.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn sign_out_without_a_session_is_401(database: PgPool) {
    let signed_out = sign_out(&app(database), None).await;

    assert_eq!(signed_out.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(body_json(signed_out).await["code"], "not_signed_in");
}

#[sqlx::test]
async fn rule_3_2_3_after_a_sign_in_the_previous_cookie_is_refused(database: PgPool) {
    let app = app(database);
    let before = signed_in_cookie(&app).await;

    let after = cookie_of(&sign_in_as_pat(&app, Some(&before)).await);

    assert_ne!(after, before);
    assert_eq!(who_am_i(&app, Some(&before)).await.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(who_am_i(&app, Some(&after)).await.status(), StatusCode::OK);
}

/// Two routers stand for two deploys with different configuration, serving one session.
/// No deploy carries the configuration yet; plan item P3.10 closes spec rule 3.3.4 on the deployed app.
#[sqlx::test]
async fn rule_3_3_4_admin_status_is_evaluated_on_each_request(database: PgPool) {
    let pat_is_an_admin = app_with_admins(database.clone(), AdminEmails::new(["pat@example.com"]));
    let pat_is_no_admin = app_with_admins(database, AdminEmails::none());
    let cookie = signed_in_cookie(&pat_is_an_admin).await;

    let under_the_first = body_json(who_am_i(&pat_is_an_admin, Some(&cookie)).await).await;
    let under_the_second = body_json(who_am_i(&pat_is_no_admin, Some(&cookie)).await).await;

    assert_eq!(under_the_first["is_admin"], true);
    assert_eq!(under_the_second["is_admin"], false);
}

#[test]
fn rule_4_6_1_the_fake_identity_route_is_not_an_api_operation() {
    let document = openapi().to_json().unwrap();

    assert!(document.contains("/api/session"));
    assert!(!document.contains("fake"));
}

/// The probe proves the route's absence only if it asks for the path the route really has.
#[test]
fn rule_2_3_5_the_image_probe_asks_for_the_fake_sign_in_by_its_real_path() {
    let probe = include_str!("../../../scripts/smoke-image.sh");

    assert!(probe.contains(&format!("localhost:$port{FAKE_SIGN_IN}\"")));
}
