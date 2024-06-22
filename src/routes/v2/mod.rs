use axum::{routing::post, Router};

pub mod claim;

pub fn create_router() -> Router {
    Router::new().route("/claim", post(claim::claim_tokens))
}
