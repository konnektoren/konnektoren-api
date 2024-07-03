use crate::storage::ProfileRepository;
use axum::{routing::post, Router};
use std::sync::Arc;

pub mod claim;

pub fn create_router() -> Router<Arc<dyn ProfileRepository>> {
    let router = Router::new();

    #[cfg(feature = "ton")]
    let router = router.route("/claim", post(claim::claim_tokens));

    router
}
