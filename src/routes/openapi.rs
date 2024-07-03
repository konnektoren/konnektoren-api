use super::{v1, v2};
use utoipa::OpenApi;

/// Documentation for the API.
#[cfg(feature = "ton")]
#[derive(OpenApi)]
#[openapi(
    // List of API endpoints to be included in the documentation.
    paths(
        super::v1::claim::claim_tokens,
        super::v2::claim::claim_tokens,
    ),
    // Schema components for requests and responses used across the API.
    components(
        schemas(
            v1::claim::ClaimRequest,
            v2::claim::ClaimV2Request,
            v2::claim::ClaimV2Response,
        )
    ),
    tags(
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
        super::v1::profile::post_profile
    ),
// Schema components for requests and responses used across the API.
    components(
        schemas(
            v1::profile::ProfileV1Response
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
    }
}
