//! The `gold-ballot` binary: serves the app, or prints its OpenAPI document.

use std::error::Error;
use std::net::{Ipv4Addr, SocketAddr};
use std::process::ExitCode;

use std::sync::Arc;

use adapters::clock::SystemClock;
use adapters::config::ServerConfig;
use adapters::http::{self, ApiState, Spa};
use adapters::postgres;
use adapters::telemetry::Telemetry;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio::signal::unix::{SignalKind, signal};

const USAGE: &str = "usage: gold-ballot [serve | openapi]";

fn main() -> ExitCode {
    let command = std::env::args().nth(1);
    let outcome = match command.as_deref() {
        None | Some("serve") => serve(),
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

/// Runs the server until SIGTERM or SIGINT, then exports the telemetry still in memory (spec rule 2.4.4).
fn serve() -> Result<(), Box<dyn Error>> {
    let config = ServerConfig::from_env()?;
    // Telemetry starts before the runtime and stops after it, because its exporter blocks.
    let telemetry =
        Telemetry::install(&config.telemetry).map_err(|error| format!("cannot start telemetry: {error}"))?;

    let outcome = Runtime::new()?.block_on(serve_until_shutdown(config, telemetry.clone()));
    if let Err(error) = &outcome {
        tracing::error!(%error, "the server stopped with an error");
    }
    telemetry.shutdown();
    outcome
}

async fn serve_until_shutdown(config: ServerConfig, telemetry: Telemetry) -> Result<(), Box<dyn Error>> {
    let spa = config.spa_dir.map(Spa::load).transpose()?;

    let database = postgres::connect(&config.database_url)
        .await
        .map_err(|error| format!("cannot connect to the database: {error}"))?;
    postgres::migrate(&database)
        .await
        .map_err(|error| format!("cannot migrate the database: {error}"))?;

    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, config.port))).await?;
    tracing::info!(port = config.port, "listening");
    let state = ApiState::new(database.clone(), Arc::new(SystemClock), config.admin_emails);
    axum::serve(listener, http::router(spa, state, telemetry))
        .with_graceful_shutdown(shutdown_requested())
        .await?;
    tracing::info!("every in-flight request has finished");

    database.close().await;
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
