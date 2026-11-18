//! Serves the built single-page app: its files, and its shell for every path that is not a file.

use std::io;
use std::path::{Path, PathBuf};

use axum::Router;
use axum::body::Bytes;
use axum::handler::HandlerWithoutStateExt;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::http::{HeaderValue, Response};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;

/// The fallback page that SvelteKit's static adapter writes; it boots the app on any route.
const SHELL_FILE: &str = "200.html";
/// SvelteKit puts content-hashed files here, so a given path never changes its content.
const HASHED_ASSETS_PATH: &str = "/_app/immutable";

const NEVER_REUSE_WITHOUT_REVALIDATION: HeaderValue = HeaderValue::from_static("no-cache");
const REUSE_FOREVER: HeaderValue = HeaderValue::from_static("public, max-age=31536000, immutable");

/// A built SPA on disk, with its shell held in memory.
pub struct Spa {
    dir: PathBuf,
    shell: Bytes,
}

impl Spa {
    /// Loads the SPA built into `dir`. Fails if the shell is missing, so a broken image cannot start.
    pub fn load(dir: impl Into<PathBuf>) -> io::Result<Self> {
        let dir = dir.into();
        let shell_path = dir.join(SHELL_FILE);
        let shell = std::fs::read(&shell_path).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("cannot read the SPA shell at {}: {error}", shell_path.display()),
            )
        })?;
        Ok(Self {
            dir,
            shell: shell.into(),
        })
    }

    /// Routes for every non-API path.
    ///
    /// Hashed assets are cached immutably, and a missing one is a 404 rather than the shell.
    /// The shell is never reused without revalidation, so a deploy reaches open browsers (spec rule 2.4.7).
    pub(super) fn into_router(self) -> Router {
        let shell = self.shell;
        let serve_shell = move || async move {
            (
                [
                    (CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8")),
                    (CACHE_CONTROL, NEVER_REUSE_WITHOUT_REVALIDATION),
                ],
                shell,
            )
        };

        let hashed_assets = ServeDir::new(hashed_assets_dir(&self.dir));
        let cache_found_assets_forever = SetResponseHeaderLayer::overriding(CACHE_CONTROL, |response: &Response<_>| {
            response.status().is_success().then_some(REUSE_FOREVER)
        });

        Router::new()
            .nest_service(HASHED_ASSETS_PATH, hashed_assets)
            .layer(cache_found_assets_forever)
            .fallback_service(ServeDir::new(self.dir).fallback(serve_shell.into_service()))
    }
}

fn hashed_assets_dir(spa_dir: &Path) -> PathBuf {
    spa_dir.join(HASHED_ASSETS_PATH.trim_start_matches('/'))
}
