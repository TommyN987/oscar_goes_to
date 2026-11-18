//! The `gold-ballot` binary: serves the app, or prints its OpenAPI document.

use std::error::Error;
use std::net::{Ipv4Addr, SocketAddr};
use std::process::ExitCode;

use adapters::config::ServerConfig;
use adapters::http::{self, Spa};
use tokio::net::TcpListener;
use tokio::signal::unix::{SignalKind, signal};

const USAGE: &str = "usage: gold-ballot [serve | openapi]";

#[tokio::main]
async fn main() -> ExitCode {
    let command = std::env::args().nth(1);
    let outcome = match command.as_deref() {
        None | Some("serve") => serve().await,
        Some("openapi") => print_openapi(),
        Some(unknown) => Err(format!("unknown command `{unknown}`\n{USAGE}").into()),
    };
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("gold-ballot: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn serve() -> Result<(), Box<dyn Error>> {
    let config = ServerConfig::from_env()?;
    let spa = config.spa_dir.map(Spa::load).transpose()?;
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, config.port))).await?;

    axum::serve(listener, http::router(spa))
        .with_graceful_shutdown(shutdown_requested())
        .await?;
    Ok(())
}

fn print_openapi() -> Result<(), Box<dyn Error>> {
    println!("{}", http::openapi().to_pretty_json()?);
    Ok(())
}

/// Resolves on SIGTERM (Cloud Run stopping the instance) or SIGINT (Ctrl-C).
/// In-flight requests then finish before the process exits (spec rule 2.4.4).
async fn shutdown_requested() {
    let mut terminate = signal(SignalKind::terminate()).expect("a SIGTERM handler can be installed");
    let mut interrupt = signal(SignalKind::interrupt()).expect("a SIGINT handler can be installed");
    tokio::select! {
        _ = terminate.recv() => {}
        _ = interrupt.recv() => {}
    }
}
