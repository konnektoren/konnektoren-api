use axum::routing::get;
use axum::{routing::post, Router};

pub mod claim;
pub mod profile;

pub fn create_router() -> Router {
    let router = Router::new();

    let router = router.route("/profiles/:profile_id", get(profile::get_profile));
    let router = router.route("/profiles", post(profile::post_profile));

    #[cfg(feature = "ton")]
    let router = router.route("/claim", post(claim::claim_tokens));

    router
}
