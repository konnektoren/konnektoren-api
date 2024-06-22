use axum::Router;
use dotenv::dotenv;
use konnektoren_api::routes;
use routes::openapi::ApiDoc;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    dotenv().ok();

    let app = Router::new()
        .nest("/api/v1", routes::v1::create_router())
        .nest("/api/v2", routes::v2::create_router())
        .layer(CorsLayer::permissive())
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    log::info!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
