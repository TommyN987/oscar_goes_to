//! Telemetry: structured logs on stdout, and traces, logs and metrics exported over OTLP.
//!
//! This is the only module that names OpenTelemetry types (spec rule 2.7.1).
//! Everything else uses `tracing` macros and the [`Telemetry`] handle.
//!
//! The OTLP exporters read the standard `OTEL_EXPORTER_OTLP_*` variables themselves,
//! so changing the backend is a change of endpoint and headers.

use std::error::Error;
use std::time::Duration;

use opentelemetry::KeyValue;
use opentelemetry::metrics::{Histogram, MeterProvider};
use opentelemetry::trace::{Status, TracerProvider};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, MetricExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing::Level;
use tracing::level_filters::LevelFilter;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::Layer;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::config::TelemetryConfig;

const SERVICE_NAME: &str = "gold-ballot";

/// Marks the span the caller is in as failed, so a trace shows which step it was and why.
/// `description` must hold no personal data (spec rule 2.7.6). Without an exporter this does nothing.
pub fn mark_current_span_failed(description: &str) {
    tracing::Span::current().set_status(Status::error(description.to_owned()));
}

/// The longest one export may take. Shutdown flushes three signals in turn and has 10 s on Cloud Run,
/// so a backend that never answers costs 6 s at most.
const EXPORT_TIMEOUT: Duration = Duration::from_secs(2);

/// Crates the exporter itself runs on. Exporting their events would create more events to export.
const EXPORTER_INTERNAL_TARGETS: [&str; 4] = ["opentelemetry", "reqwest", "hyper", "h2"];

/// Bucket boundaries, in seconds, for request durations.
const REQUEST_DURATION_BOUNDARIES: [f64; 11] = [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0];

/// The handle through which the rest of the server records and flushes telemetry.
#[derive(Clone)]
pub struct Telemetry {
    exporting: Option<Providers>,
    flush_each_request: bool,
    request_duration: Histogram<f64>,
}

#[derive(Clone)]
struct Providers {
    traces: SdkTracerProvider,
    logs: SdkLoggerProvider,
    metrics: SdkMeterProvider,
}

impl Telemetry {
    /// Installs the process-wide `tracing` subscriber. Call it once, before the async runtime starts.
    ///
    /// Events at INFO and above print as JSON on stdout.
    /// If `config.export` is set, spans, events and metrics are also batched and exported over OTLP.
    /// An event inside a span is exported with that span's trace ID, because the span layer makes
    /// its span the current OpenTelemetry context (spec rule 2.7.3).
    /// Export runs on background threads, so an unreachable backend never fails or slows a request
    /// (spec rule 2.7.7).
    pub fn install(config: &TelemetryConfig) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let exporting = if config.export {
            Some(Providers::exporting_over_otlp()?)
        } else {
            None
        };

        let stdout = tracing_subscriber::fmt::layer().json().with_filter(LevelFilter::INFO);
        let spans = exporting.as_ref().map(|providers| {
            tracing_opentelemetry::layer()
                .with_tracer(providers.traces.tracer(SERVICE_NAME))
                .with_filter(exported_events())
        });
        let logs = exporting
            .as_ref()
            .map(|providers| OpenTelemetryTracingBridge::new(&providers.logs).with_filter(exported_events()));
        tracing_subscriber::registry()
            .with(stdout)
            .with(spans)
            .with(logs)
            .try_init()?;

        let request_duration = match &exporting {
            Some(providers) => request_duration_histogram(&providers.metrics),
            None => request_duration_histogram(&SdkMeterProvider::default()),
        };
        Ok(Self {
            exporting,
            flush_each_request: config.flush_each_request,
            request_duration,
        })
    }

    /// A handle that records nothing and installs nothing, for tests of the router.
    pub fn disabled() -> Self {
        Self {
            exporting: None,
            flush_each_request: false,
            request_duration: request_duration_histogram(&SdkMeterProvider::default()),
        }
    }

    /// Records one finished request by method, route template and status (spec rule 2.7.4).
    /// `route` must be a template such as `/api/health`, never a raw path, which could hold personal data.
    pub fn record_request(&self, method: &str, route: &str, status: u16, duration: Duration) {
        self.request_duration.record(
            duration.as_secs_f64(),
            &[
                KeyValue::new("http.request.method", method.to_owned()),
                KeyValue::new("http.route", route.to_owned()),
                KeyValue::new("http.response.status_code", i64::from(status)),
            ],
        );
    }

    /// Whether telemetry is exported before each response instead of in the background (spec rule 2.7.5).
    pub fn flushes_each_request(&self) -> bool {
        self.flush_each_request
    }

    /// Exports everything recorded so far. A failed export is dropped, never reported to the caller.
    pub async fn flush(&self) {
        if let Some(providers) = self.exporting.clone() {
            // Exporting blocks, so it must not run on an async worker thread.
            let _ = tokio::task::spawn_blocking(move || providers.flush()).await;
        }
    }

    /// Exports everything recorded so far and stops exporting. Call it after the async runtime has stopped.
    pub fn shutdown(self) {
        if let Some(providers) = self.exporting {
            let _ = providers.traces.shutdown();
            let _ = providers.logs.shutdown();
            let _ = providers.metrics.shutdown();
        }
    }
}

impl Providers {
    fn exporting_over_otlp() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let resource = Resource::builder().with_service_name(SERVICE_NAME).build();

        let traces = SdkTracerProvider::builder()
            .with_resource(resource.clone())
            .with_batch_exporter(
                SpanExporter::builder()
                    .with_http()
                    .with_timeout(EXPORT_TIMEOUT)
                    .build()?,
            )
            .build();
        let logs = SdkLoggerProvider::builder()
            .with_resource(resource.clone())
            .with_batch_exporter(
                LogExporter::builder()
                    .with_http()
                    .with_timeout(EXPORT_TIMEOUT)
                    .build()?,
            )
            .build();
        let metrics = SdkMeterProvider::builder()
            .with_resource(resource)
            .with_periodic_exporter(
                MetricExporter::builder()
                    .with_http()
                    .with_timeout(EXPORT_TIMEOUT)
                    .build()?,
            )
            .build();

        Ok(Self { traces, logs, metrics })
    }

    fn flush(&self) {
        let _ = self.traces.force_flush();
        let _ = self.logs.force_flush();
        let _ = self.metrics.force_flush();
    }
}

/// What leaves the process: INFO and above, minus the exporter's own internals.
fn exported_events() -> Targets {
    EXPORTER_INTERNAL_TARGETS
        .iter()
        .fold(Targets::new().with_default(Level::INFO), |targets, internal| {
            targets.with_target(*internal, LevelFilter::OFF)
        })
}

fn request_duration_histogram(metrics: &SdkMeterProvider) -> Histogram<f64> {
    metrics
        .meter(SERVICE_NAME)
        .f64_histogram("http.server.request.duration")
        .with_unit("s")
        .with_description("Duration of HTTP requests, by method, route and status.")
        .with_boundaries(REQUEST_DURATION_BOUNDARIES.to_vec())
        .build()
}
