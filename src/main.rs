use axum::Router;
use dotenv::dotenv;
use konnekt_session::server::v2::{create_session_route, ConnectionHandler, MemoryStorage};
use konnektoren_api::routes;
use konnektoren_api::storage::Storage;
use routes::openapi::ApiDoc;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    dotenv().ok();

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
        .with_state(repo)
        .layer(CorsLayer::permissive())
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()));

    #[cfg(feature = "konnekt-session")]
    let app = app.merge(create_session_route(session_server));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    log::info!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
