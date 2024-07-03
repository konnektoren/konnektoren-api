use axum::handler::Handler;
use axum::Router;
use dotenv::dotenv;
use konnektoren_api::routes;
use konnektoren_api::storage::{ProfileRepository, RedisStorage};
use routes::openapi::ApiDoc;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    dotenv().ok();

    // Create RedisProfileRepository
    let repo: Arc<dyn ProfileRepository> = Arc::new(RedisStorage::new("redis://127.0.0.1/"));

    let app = Router::new()
        .nest("/api/v1", routes::v1::create_router())
        .nest("/api/v2", routes::v2::create_router())
        .with_state(repo)
        .layer(CorsLayer::permissive())
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    log::info!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
