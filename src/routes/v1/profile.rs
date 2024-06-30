use crate::services::v1::profile::{fetch_profile, save_profile};
use axum::extract::Path;
use axum::Json;
use konnektoren_core::prelude::PlayerProfile;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ProfileV1Response {
    #[schema()]
    pub profile: Option<PlayerProfile>,
}

pub type ProfileV1Request = PlayerProfile;

fn profile_example() -> PlayerProfile {
    PlayerProfile::new("example_user_id".to_string())
}

#[utoipa::path(
    get,
    operation_id = "get_profile_v1",
    tag = "profile_v1",
    path = "/profiles/{profile_id}",
    params(
        ("profile_id", description = "Id for the profile to be retrieved"),
    ),
    context_path = "/api/v1",
    responses(
    (status = 200, description = "Profile loaded successfully", body = ProfileV1Response),
    (status = 400, description = "Invalid request data"),
    )
)]
pub async fn get_profile(
    Path(profile_id): Path<String>,
) -> Result<Json<ProfileV1Response>, (axum::http::StatusCode, String)> {
    let profile = fetch_profile(profile_id).await.map_err(|err| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
    })?;
    Ok(Json(ProfileV1Response {
        profile: Some(profile),
    }))
}

#[utoipa::path(
    post,
    operation_id = "post_profile_v1",
    tag = "profile_v1",
    path = "/profiles",
    context_path = "/api/v1",
    request_body(content = ProfileV1Request, example = json!(profile_example())),
    responses(
        (status = 200, description = "Profile successfully saved", body = ProfileV1Response),
        (status = 400, description = "Invalid request data"),
    )
)]
pub async fn post_profile(
    Json(profile): Json<PlayerProfile>,
) -> Result<Json<PlayerProfile>, (axum::http::StatusCode, String)> {
    let profile = save_profile(profile).await.map_err(|err| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
    })?;
    Ok(Json(profile))
}
