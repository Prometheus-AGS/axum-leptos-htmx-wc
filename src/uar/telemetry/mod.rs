use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize application telemetry (Logging, Tracing, Metrics).
///
/// Currently configures:
/// - `tracing-subscriber::fmt` for structured logging.
/// - `EnvFilter` for dynamic log levels (RUST_LOG).
///
/// Future:
/// - OpenTelemetry layer for distributed tracing.
/// - Prometheus exporter for metrics.
pub fn init() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .compact();

    let filter_layer = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,axum_leptos_htmx_wc=debug"));

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        // .with(opentelemetry_layer) // TODO: Add OTel here
        .init();
}
