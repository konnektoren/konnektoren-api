use std::env;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

#[cfg(feature = "tracing")]
use {
    opentelemetry::{
        global,
        sdk::{
            propagation::TraceContextPropagator,
            trace::{self, RandomIdGenerator, Sampler},
            Resource,
        },
        KeyValue,
    },
    tracing_opentelemetry,
};

pub async fn init_telemetry() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "tracing")]
    let enable_telemetry = env::var("ENABLE_TELEMETRY")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::default())
        .add_directive("isahc=off".parse().unwrap())
        .add_directive("tower_http=info".parse().unwrap());

    // Create a more readable console format
    let fmt_layer = fmt::layer()
        .with_target(false) // Hide target
        .with_thread_ids(false)
        .with_line_number(false) // Hide line numbers
        .with_file(false) // Hide file paths
        .with_ansi(true)
        .with_level(true)
        .with_timer(fmt::time::UtcTime::new(time::format_description::parse(
            "[year]-[month]-[day]T[hour]:[minute]:[second]Z",
        )?))
        .compact();

    #[cfg(feature = "tracing")]
    if enable_telemetry {
        global::set_text_map_propagator(TraceContextPropagator::new());
        let jaeger_endpoint = env::var("JAEGER_ENDPOINT")
            .unwrap_or_else(|_| "http://jaeger:14268/api/traces".to_string());
        let tracer = opentelemetry_jaeger::new_collector_pipeline()
            .with_service_name(env!("CARGO_PKG_NAME"))
            .with_endpoint(jaeger_endpoint)
            .with_reqwest()
            .with_trace_config(
                trace::config()
                    .with_sampler(Sampler::AlwaysOn)
                    .with_id_generator(RandomIdGenerator::default())
                    .with_max_events_per_span(64)
                    .with_max_attributes_per_span(16)
                    .with_resource(Resource::new(vec![
                        KeyValue::new("service.name", env!("CARGO_PKG_NAME")),
                        KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                    ])),
            )
            .with_timeout(std::time::Duration::from_secs(2))
            .install_batch(opentelemetry::runtime::Tokio)?;

        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

        // Create separate JSON layer for telemetry
        let telemetry_fmt = fmt::layer()
            .json()
            .with_current_span(true)
            .with_span_list(true)
            .with_target(true)
            .with_thread_ids(true)
            .with_line_number(true)
            .with_file(true)
            .with_writer(std::io::stderr); // Write JSON logs to stderr

        Registry::default()
            .with(env_filter)
            .with(fmt_layer) // Console-friendly format
            .with(telemetry_fmt) // JSON format for telemetry
            .with(telemetry)
            .try_init()?;

        tracing::info!("Telemetry enabled with Jaeger");
        return Ok(());
    }

    // Default initialization without tracing
    Registry::default()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()?;

    #[cfg(feature = "tracing")]
    tracing::info!("Telemetry disabled");
    #[cfg(not(feature = "tracing"))]
    tracing::info!("Tracing feature not enabled");

    Ok(())
}
