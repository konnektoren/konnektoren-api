use crate::metrics::Metrics;
use axum::{
    body::Body,
    extract::{MatchedPath, State},
    middleware::Next,
    response::Response,
};
use opentelemetry::KeyValue;
use std::time::Instant;

pub async fn metrics_middleware(
    State(metrics): State<Metrics>,
    request: axum::http::Request<Body>,
    next: Next,
) -> Response {
    let start = Instant::now();

    let path = request
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str)
        .unwrap_or("unknown")
        .to_string();

    let method = request.method().to_string();

    metrics.request_counter.add(
        1,
        &[
            KeyValue::new("path", path.clone()),
            KeyValue::new("method", method.clone()),
        ],
    );

    let response = next.run(request).await;

    let duration = start.elapsed().as_secs_f64();
    metrics.request_duration.record(
        duration,
        &[
            KeyValue::new("path", path),
            KeyValue::new("method", method),
            KeyValue::new("status", response.status().as_u16() as i64),
        ],
    );

    response
}
