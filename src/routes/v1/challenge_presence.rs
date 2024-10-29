use crate::storage::Storage;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ChallengePresenceStats {
    pub count: u32,
}

#[utoipa::path(
    get,
    operation_id = "get_challenge_presence",
    tag = "challenge_presence",
    path = "/challenges/{challenge_id}/presence",
    params(
        ("challenge_id", description = "Challenge ID to get active participants count"),
    ),
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Active participants count retrieved successfully", body = ChallengePresenceStats),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn get_challenge_presence(
    State(repository): State<Arc<Mutex<dyn Storage>>>,
    Path(challenge_id): Path<String>,
) -> Result<Json<ChallengePresenceStats>, (StatusCode, String)> {
    let repo = repository.lock().await;
    let count = repo
        .get_active_count(&challenge_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ChallengePresenceStats { count }))
}

#[utoipa::path(
    post,
    operation_id = "record_challenge_presence",
    tag = "challenge_presence",
    path = "/challenges/{challenge_id}/presence/record",
    params(
        ("challenge_id", description = "Challenge ID to record presence for"),
    ),
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Presence recorded successfully", body = ChallengePresenceStats),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn record_challenge_presence(
    State(repository): State<Arc<Mutex<dyn Storage>>>,
    Path(challenge_id): Path<String>,
) -> Result<Json<ChallengePresenceStats>, (StatusCode, String)> {
    let mut repo = repository.lock().await;
    let count = repo
        .record_presence(&challenge_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ChallengePresenceStats { count }))
}
