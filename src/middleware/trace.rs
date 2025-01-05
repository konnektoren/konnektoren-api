use axum::{
    extract::{MatchedPath, Request},
    middleware::Next,
    response::Response,
};
use http::header::HeaderName;
use opentelemetry::trace::{SpanContext, SpanId, TraceContextExt, TraceFlags, TraceId};
use opentelemetry::Context;
use tracing::{info_span, Instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use uuid::Uuid;

pub async fn trace_middleware(request: Request, next: Next) -> Response {
    // 1. Extract or create a Context (trace_id, parent_span_id) for this request
    let parent_context = extract_or_create_session_context(&request);

    // Log the incoming traceparent for debugging
    if let Some(traceparent) = request.headers().get("traceparent") {
        tracing::debug!("Incoming traceparent: {:?}", traceparent);
    }

    // 2. Optionally, extract the session ID (useful for tagging/logging)
    let session_id = request
        .headers()
        .get("X-Session-ID")
        .and_then(|header| header.to_str().ok())
        .unwrap_or_default();

    // 3. Extract the matched path for logging
    let matched_path = request
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str)
        .unwrap_or_default();

    // 4. Create a new tracing span for this request.
    //    We add the session_id and matched_path as attributes (labels) for convenience.
    let span = info_span!(
        "http_request",
        trace_id = %parent_context.span().span_context().trace_id(),
        parent_span_id = %parent_context.span().span_context().span_id(),
        method = %request.method(),
        uri = %request.uri(),
        session_id = %session_id,
        matched_path = %matched_path
    );

    // 5. Set the parent of this new span to the extracted/created context.
    //    This means the new span will share the parent's Trace ID; if there was no parent,
    //    it becomes a root-like span for that session’s Trace ID.
    span.set_parent(parent_context.clone());

    // We instrument our async block with that new span.
    let span_clone = span.clone();
    async move {
        // 6. Run the request
        let response = next.run(request).await;

        // 7. Retrieve the current span’s context (the new child span we made)
        let current_context = span_clone.context();
        let current_span = current_context.span();
        let current_span_context = current_span.span_context();

        // 8. Construct a new traceparent header for outgoing response or logs
        let trace_id = current_span_context.trace_id().to_string();
        let span_id = current_span_context.span_id().to_string();
        let traceparent = format!("00-{}-{}-01", trace_id, span_id);
        tracing::debug!("Outgoing traceparent: {}", traceparent);

        let mut response = response;
        response.headers_mut().insert(
            HeaderName::from_static("traceparent"),
            traceparent.parse().unwrap(),
        );

        response
    }
    .instrument(span)
    .await
}

/// Extract the trace context from the `traceparent` header if present. Otherwise:
///   - Retrieve (or generate) a session ID from `X-Session-ID`.
///   - Use that session ID to form the trace_id.
///   - Use 0 as a “parent” span_id (so each request is a sibling at the top level of that trace).
///   - Generate a new random span_id as well, although typically we generate the actual request’s
///     span_id only when we create the new child span.
fn extract_or_create_session_context(request: &Request) -> Context {
    // If the request has a valid traceparent, parse that and continue that trace.
    if let Some(traceparent) = request.headers().get("traceparent") {
        if let Ok(traceparent_str) = traceparent.to_str() {
            let parts: Vec<&str> = traceparent_str.split('-').collect();
            if parts.len() >= 4 {
                if let (Ok(trace_id), Ok(span_id)) = (
                    TraceId::from_hex(parts[1]),
                    SpanId::from_hex(&parts[2][..16]),
                ) {
                    let span_context = SpanContext::new(
                        trace_id,
                        span_id,
                        TraceFlags::SAMPLED,
                        true,
                        Default::default(),
                    );
                    tracing::debug!(
                        "Continuing existing trace: trace_id={}, parent_span_id={}",
                        trace_id,
                        span_id
                    );
                    return Context::new().with_remote_span_context(span_context);
                }
            }
        }
    }

    // If there's no valid traceparent, we fall back to using the session ID as the trace ID.
    // 1. Extract or generate a session ID
    let session_id = request
        .headers()
        .get("X-Session-ID")
        .and_then(|header| header.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            let id = Uuid::new_v4().to_string().replace('-', "");
            tracing::debug!("Generated new session ID: {}", id);
            id
        });

    // 2. Convert the session ID into a valid TraceId, or generate a random one if conversion fails
    let trace_id = match TraceId::from_hex(&session_id) {
        Ok(tid) => {
            tracing::debug!("Using session ID as trace ID: {}", tid);
            tid
        }
        Err(_) => {
            // If session_id isn't a valid 32-char hex, generate a new random trace ID
            let fallback = Uuid::new_v4().to_string().replace("-", "");
            let tid = TraceId::from_hex(&fallback).unwrap();
            tracing::debug!(
                "Session ID was not a valid hex trace_id. Fallback trace_id={}",
                tid
            );
            tid
        }
    };

    // 3. Define the parent span ID as zero (all requests for this session become siblings)
    let parent_span_id = SpanId::from(0);

    // 4. Create a remote span context so we can treat it as parent
    let span_context = SpanContext::new(
        trace_id,
        parent_span_id,
        TraceFlags::SAMPLED,
        true,
        Default::default(),
    );

    tracing::debug!(
        "Created new parent context from session: trace_id={}, parent_span_id={}",
        trace_id,
        parent_span_id
    );

    // 5. Return a new Context with that remote span context
    Context::new().with_remote_span_context(span_context)
}
