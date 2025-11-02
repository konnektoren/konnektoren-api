use super::{v1, v2};
use utoipa::OpenApi;

/// Documentation for the API.
#[cfg(feature = "ton")]
#[derive(OpenApi)]
#[openapi(
    // List of API endpoints to be included in the documentation.
    paths(
        super::health::health_check,
        super::health::readiness_check,
        super::v1::claim::claim_tokens,
        super::v2::claim::claim_tokens,
    ),
    // Schema components for requests and responses used across the API.
    components(
        schemas(
            super::health::HealthResponse,
            super::health::ReadinessResponse,
            super::health::HealthChecks,
            v1::claim::ClaimRequest,
            v2::claim::ClaimV2Request,
            v2::claim::ClaimV2Response,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "API", description = "Konnektoren API")
    )
)]
pub struct ApiDoc;

/// Documentation for the API.
#[cfg(not(feature = "ton"))]
#[derive(OpenApi)]
#[openapi(
// List of API endpoints to be included in the documentation.
    paths(
        super::v1::profile::get_profile,
        super::v1::profile::get_all_profiles,
        super::v1::profile::post_profile,
        super::v1::leaderboard::get_leaderboard,
        super::v1::leaderboard::get_challenge_leaderboard,
        super::v1::leaderboard::post_performance_record,
        super::v1::leaderboard::post_challenge_performance_record,
        super::v1::review::get_reviews,
        super::v1::review::post_review,
        super::v1::review::get_all_reviews,
        super::v1::review::get_average_rating,
        super::v1::challenge_presence::get_challenge_presence,
        super::v1::challenge_presence::record_challenge_presence,
        // Coupon endpoints
        super::v1::coupon::create_handler,
        super::v1::coupon::get_handler,
        super::v1::coupon::list_handler,
        super::v1::coupon::validate_handler,
        super::v1::coupon::redeem_handler,
        #[cfg(feature = "chat")]
        super::v1::chat::send_message,
        #[cfg(feature = "chat")]
        super::v1::chat::receive_messages,

    ),
// Schema components for requests and responses used across the API.
    components(
        schemas(
            v1::profile::ProfileV1Response,
            v1::profile::ProfilesV1Response,
            v1::leaderboard::LeaderboardV1Response,
            v1::review::Review,
            v1::review::ReviewsResponse,
            v1::challenge_presence::ChallengePresenceStats,
            v1::coupon::CouponResponse,
            v1::coupon::CouponsResponse,
        )
    ),
    tags(
        (name = "API", description = "Konnektoren API")
    )
)]
pub struct ApiDoc;

#[cfg(test)]
mod tests {
    use super::*;
    use utoipa::OpenApi;

    #[test]
    fn api_doc_contains_all_endpoints() {
        let api_doc = ApiDoc::openapi();
        let paths = api_doc.paths.paths;
        assert!(paths.contains_key("/api/v1/profiles/{profile_id}"));
        assert!(paths.contains_key("/api/v1/profiles"));
        assert!(paths.contains_key("/api/v1/challenges/{challenge_id}/presence"));
        assert!(paths.contains_key("/api/v1/challenges/{challenge_id}/presence/record"));

        // Coupon endpoint tests
        assert!(paths.contains_key("/api/v1/coupons"));
        assert!(paths.contains_key("/api/v1/coupons/{code}"));
        assert!(paths.contains_key("/api/v1/coupons/{code}/validate/{challenge_id}"));
        assert!(paths.contains_key("/api/v1/coupons/{code}/redeem/{challenge_id}"));

        #[cfg(feature = "chat")]
        {
            assert!(paths.contains_key("/api/v1/chat/send/{channel}"));
            assert!(paths.contains_key("/api/v1/chat/receive/{channel}"));
        }
    }
}
