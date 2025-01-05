use axum::{
    extract::{MatchedPath, Request},
    Router,
};
use dotenv::dotenv;
use konnekt_session::server::v2::{create_session_route, ConnectionHandler, MemoryStorage};
use konnektoren_api::{routes, storage::Storage, telemetry::init_telemetry};
use routes::openapi::ApiDoc;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
#[cfg(feature = "tracing")]
use tower_http::trace::{self, TraceLayer};
#[cfg(feature = "tracing")]
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    dotenv().ok();
    init_telemetry()
        .await
        .expect("Failed to initialize telemetry");

    #[cfg(feature = "redis")]
    let repo: Arc<Mutex<dyn Storage>> = {
        let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL env must be set");
        Arc::new(Mutex::new(konnektoren_api::storage::RedisStorage::new(
            &redis_url,
        )))
    };

    #[cfg(not(feature = "redis"))]
    let repo: Arc<Mutex<dyn Storage>> =
        Arc::new(Mutex::new(konnektoren_api::storage::MemoryRepository::new()));

    #[cfg(feature = "konnekt-session")]
    let session_server = {
        let memory_storage = Arc::new(MemoryStorage::new());
        ConnectionHandler::new(memory_storage.clone(), memory_storage.clone())
    };

    let app = Router::new()
        .nest("/api/v1", routes::v1::create_router())
        .nest("/api/v2", routes::v2::create_router())
        .with_state(repo);

    #[cfg(feature = "tracing")]
    let app = app.layer(
        TraceLayer::new_for_http()
            .make_span_with(|request: &Request<_>| {
                let matched_path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str);

                tracing::info_span!(
                    "http_request",
                    method = ?request.method(),
                    matched_path,
                    uri = ?request.uri(),
                    version = ?request.version(),
                )
            })
            .on_request(trace::DefaultOnRequest::new().level(Level::INFO))
            .on_response(trace::DefaultOnResponse::new().level(Level::INFO))
            .on_failure(trace::DefaultOnFailure::new().level(Level::ERROR)),
    );

    let app = app
        .layer(CorsLayer::permissive())
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()));

    #[cfg(feature = "konnekt-session")]
    let app = app.merge(create_session_route(session_server));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    log::info!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
