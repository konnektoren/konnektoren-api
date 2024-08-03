use crate::storage::{LeaderboardRepository, ProfileRepository, Storage};
use axum::handler::Handler;
use axum::routing::get;
use axum::{routing::post, Router};
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod claim;
pub mod leaderboard;
pub mod profile;

pub fn create_router() -> Router<Arc<Mutex<dyn Storage>>> {
    let router = Router::new();

    let router = router.route("/profiles/:profile_id", get(profile::get_profile));
    let router = router.route("/profiles", get(profile::get_all_profiles));
    let router = router.route("/profiles", post(profile::post_profile));

    #[cfg(feature = "ton")]
    let router = router.route("/claim", post(claim::claim_tokens));

    let router = router.route("/leaderboard", get(leaderboard::get_leaderboard));
    let router = router.route(
        "/performance-record",
        post(leaderboard::post_performance_record),
    );
    let router = router.route(
        "/leaderboard/:challenge_id",
        get(leaderboard::get_challenge_leaderboard),
    );
    let router = router.route(
        "/performance-record/:challenge_id",
        post(leaderboard::post_challenge_performance_record),
    );

    router
}
