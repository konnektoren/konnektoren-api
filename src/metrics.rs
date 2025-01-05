use opentelemetry::{
    metrics::{Meter, MeterProvider as _},
    KeyValue,
};
use opentelemetry_otlp::{ExportConfig, Protocol, WithExportConfig};
use opentelemetry_sdk::{
    metrics::{MeterProvider, PeriodicReader},
    runtime::Tokio,
};
use std::env;
use std::{sync::Arc, time::Duration};

#[derive(Clone)]
pub struct Metrics {
    pub meter: Meter,
    pub request_counter: Arc<opentelemetry::metrics::Counter<u64>>,
    pub request_duration: Arc<opentelemetry::metrics::Histogram<f64>>,
}

impl Metrics {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let jaeger_endpoint = env::var("JAEGER_METRICS_ENDPOINT")
            .unwrap_or_else(|_| "http://jaeger:4318/v1/metrics".to_string());

        let export_config = ExportConfig {
            endpoint: jaeger_endpoint,
            protocol: Protocol::HttpBinary,
            ..ExportConfig::default()
        };

        let meter_provider = opentelemetry_otlp::new_pipeline()
            .metrics(Tokio)
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .http()
                    .with_export_config(export_config),
            )
            .with_period(Duration::from_secs(10))
            .build()?;

        let meter = meter_provider.meter("konnektoren");

        // Create metrics
        let request_counter = meter
            .u64_counter("http_requests_total")
            .with_description("Total number of HTTP requests")
            .init();

        let request_duration = meter
            .f64_histogram("http_request_duration_seconds")
            .with_description("HTTP request duration in seconds")
            .init();

        Ok(Metrics {
            meter,
            request_counter: Arc::new(request_counter),
            request_duration: Arc::new(request_duration),
        })
    }
}
