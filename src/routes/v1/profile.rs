use crate::services::v1::profile::{fetch_all_profiles, fetch_profile, save_profile};
use crate::storage::Storage;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use konnektoren_core::prelude::PlayerProfile;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ProfileV1Response {
    #[schema()]
    pub profile: Option<PlayerProfile>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ProfilesV1Response {
    #[schema()]
    pub profiles: Vec<PlayerProfile>,
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
    State(repository): State<Arc<Mutex<dyn Storage>>>,
    Path(profile_id): Path<String>,
) -> Result<Json<ProfileV1Response>, (StatusCode, String)> {
    let profile = fetch_profile(profile_id, repository)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(Json(ProfileV1Response {
        profile: Some(profile),
    }))
}

#[utoipa::path(
    get,
    operation_id = "get_all_profiles_v1",
    tag = "profile_v1",
    path = "/profiles",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Profiles loaded successfully", body = Vec<ProfileV1Response>),
        (status = 400, description = "Invalid request data"),
    )
)]
pub async fn get_all_profiles(
    State(repository): State<Arc<Mutex<dyn Storage>>>,
) -> Result<Json<ProfilesV1Response>, (StatusCode, String)> {
    let profiles = fetch_all_profiles(repository)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(Json(ProfilesV1Response { profiles }))
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
    State(repository): State<Arc<Mutex<dyn Storage>>>,
    Json(profile): Json<PlayerProfile>,
) -> Result<Json<PlayerProfile>, (StatusCode, String)> {
    let profile = save_profile(profile, repository)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(Json(profile))
}
