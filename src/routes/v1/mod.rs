use crate::storage::{LeaderboardRepository, ProfileRepository, Storage};
use axum::handler::Handler;
use axum::routing::get;
use axum::{routing::post, Router};
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod challenge_presence;
pub mod claim;
pub mod leaderboard;
pub mod profile;
pub mod review;

#[cfg(feature = "chat")]
pub mod chat;

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
    let router = router.route(
        "/reviews/:challenge_id/average",
        get(review::get_average_rating),
    );
    let router = router.route("/reviews/:challenge_id", get(review::get_reviews));
    let router = router.route("/reviews", post(review::post_review));
    let router = router.route("/reviews", get(review::get_all_reviews));

    #[cfg(feature = "chat")]
    let router = router.route("/chat/send/:channel", post(chat::send_message));
    #[cfg(feature = "chat")]
    let router = router.route("/chat/receive/:channel", get(chat::receive_messages));

    let router = router.route(
        "/challenges/:challenge_id/presence",
        get(challenge_presence::get_challenge_presence),
    );
    let router = router.route(
        "/challenges/:challenge_id/presence/record",
        post(challenge_presence::record_challenge_presence),
    );

    router
}
