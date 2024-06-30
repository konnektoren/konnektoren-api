use axum::{routing::post, Router};

pub mod claim;

pub fn create_router() -> Router {
    let router = Router::new();

    #[cfg(feature = "ton")]
    let router = router.route("/claim", post(claim::claim_tokens));

    router
}
