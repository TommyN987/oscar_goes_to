//! The router, called in-process without a socket.

use std::fs;
use std::path::PathBuf;

use adapters::http::{Spa, router};
use axum::Router;
use axum::body::Body;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::http::{Request, Response, StatusCode};
use http_body_util::BodyExt;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tower::ServiceExt;

const SHELL: &str = "<!doctype html><title>shell</title>";

async fn get(router: Router, path: &str) -> Response<Body> {
    let request = Request::get(path).body(Body::empty()).unwrap();
    router.oneshot(request).await.unwrap()
}

async fn body_text(response: Response<Body>) -> String {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

async fn body_json(response: Response<Body>) -> serde_json::Value {
    serde_json::from_str(&body_text(response).await).unwrap()
}

/// A pool pointed at a port where no Postgres listens. It connects only when a query runs.
fn unreachable_database() -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_millis(200))
        .connect_lazy("postgres://localhost:1/nowhere")
        .unwrap()
}

/// A built SPA with a shell, one hashed asset and one plain file, in a directory unique to `test_name`.
fn built_spa(test_name: &str) -> Spa {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(test_name);
    fs::create_dir_all(dir.join("_app/immutable")).unwrap();
    fs::write(dir.join("200.html"), SHELL).unwrap();
    fs::write(dir.join("_app/immutable/entry.abc123.js"), "export {}").unwrap();
    fs::write(dir.join("robots.txt"), "User-agent: *").unwrap();
    Spa::load(dir).unwrap()
}

#[tokio::test]
async fn rule_2_4_5_liveness_answers_without_the_database() {
    let response = get(router(None, unreachable_database()), "/api/health").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_json(response).await, serde_json::json!({ "status": "ok" }));
}

#[tokio::test]
async fn rule_2_4_5_readiness_fails_without_the_database() {
    let response = get(router(None, unreachable_database()), "/api/health/ready").await;

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body_json(response).await["code"], "database_unavailable");
}

#[sqlx::test]
async fn rule_2_4_5_readiness_answers_when_the_database_is_reachable(database: PgPool) {
    let response = get(router(None, database), "/api/health/ready").await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn rule_2_4_2_an_unknown_api_path_is_a_json_404() {
    let response = get(
        router(Some(built_spa("unknown_api_path")), unreachable_database()),
        "/api/nope",
    )
    .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(response.headers()[CONTENT_TYPE], "application/json");
    assert_eq!(body_json(response).await["code"], "not_found");
}

#[tokio::test]
async fn rule_2_4_2_an_unknown_non_api_path_is_the_shell() {
    let response = get(
        router(Some(built_spa("unknown_non_api_path")), unreachable_database()),
        "/ballot",
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[CONTENT_TYPE], "text/html; charset=utf-8");
    assert_eq!(body_text(response).await, SHELL);
}

#[tokio::test]
async fn rule_2_4_7_the_shell_is_never_cached() {
    let response = get(router(Some(built_spa("shell_cache")), unreachable_database()), "/").await;

    assert_eq!(response.headers()[CACHE_CONTROL], "no-cache");
    assert_eq!(body_text(response).await, SHELL);
}

#[tokio::test]
async fn rule_2_4_7_a_hashed_asset_is_cached_immutably() {
    let response = get(
        router(Some(built_spa("hashed_asset_cache")), unreachable_database()),
        "/_app/immutable/entry.abc123.js",
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[CACHE_CONTROL], "public, max-age=31536000, immutable");
}

#[tokio::test]
async fn a_missing_hashed_asset_is_a_404_and_is_not_cached() {
    let response = get(
        router(Some(built_spa("missing_hashed_asset")), unreachable_database()),
        "/_app/immutable/gone.js",
    )
    .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(!response.headers().contains_key(CACHE_CONTROL));
}

#[tokio::test]
async fn a_plain_file_is_served_as_itself() {
    let response = get(
        router(Some(built_spa("plain_file")), unreachable_database()),
        "/robots.txt",
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_text(response).await, "User-agent: *");
}

#[test]
fn a_spa_without_its_shell_does_not_load() {
    let empty_dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("no_shell");
    fs::create_dir_all(&empty_dir).unwrap();

    let error = Spa::load(empty_dir).err().unwrap();

    assert!(error.to_string().contains("200.html"));
}
