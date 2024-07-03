use crate::storage::ProfileRepository;
use axum::handler::Handler;
use axum::routing::get;
use axum::{routing::post, Router};
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod claim;
pub mod profile;

pub fn create_router() -> Router<Arc<Mutex<dyn ProfileRepository>>> {
    let router = Router::new();

    let router = router.route("/profiles/:profile_id", get(profile::get_profile));
    let router = router.route("/profiles", post(profile::post_profile));

    #[cfg(feature = "ton")]
    let router = router.route("/claim", post(claim::claim_tokens));

    router
}
