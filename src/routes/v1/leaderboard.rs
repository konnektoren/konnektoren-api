use crate::services::v1::leaderboard::{add_performance_record, fetch_all_performance_records};
use crate::storage::Storage;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::Utc;
use konnektoren_core::challenges::PerformanceRecord;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LeaderboardV1Response {
    #[schema()]
    pub performance_records: Vec<PerformanceRecord>,
}

fn performance_record_example() -> PerformanceRecord {
    PerformanceRecord {
        game_path_id: "example_game_path_id".to_string(),
        profile_name: "example_profile_name".to_string(),
        challenges_performance: vec![("example_challenge_id".to_string(), 100, 10000)],
        total_challenges: 1,
        performance_percentage: 100,
        date: Utc::now(),
    }
}

#[utoipa::path(
    get,
    operation_id = "get_leaderboard_v1",
    tag = "leaderboard_v1",
    path = "/leaderboard",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Leaderboard loaded successfully", body = LeaderboardV1Response),
        (status = 400, description = "Invalid request data"),
    )
)]
pub async fn get_leaderboard(
    State(repository): State<Arc<Mutex<dyn Storage>>>,
) -> Result<Json<LeaderboardV1Response>, (StatusCode, String)> {
    let namespace = "leaderboard";
    let performance_records = fetch_all_performance_records(namespace, repository)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(Json(LeaderboardV1Response {
        performance_records,
    }))
}

#[utoipa::path(
    get,
    operation_id = "get_challenge_leaderboard_v1",
    tag = "leaderboard_v1",
    path = "/leaderboard/{challenge_id}",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Leaderboard loaded successfully", body = LeaderboardV1Response),
        (status = 400, description = "Invalid request data"),
    )
)]
pub async fn get_challenge_leaderboard(
    Path(challenge_id): Path<String>,
    State(repository): State<Arc<Mutex<dyn Storage>>>,
) -> Result<Json<LeaderboardV1Response>, (StatusCode, String)> {
    let namespace = challenge_id.as_str();
    let performance_records = fetch_all_performance_records(namespace, repository)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(Json(LeaderboardV1Response {
        performance_records,
    }))
}

#[utoipa::path(
    post,
    operation_id = "post_performance_record_v1",
    tag = "leaderboard_v1",
    path = "/performance-record",
    context_path = "/api/v1",
    request_body(content = PerformanceRecord, example = json!(performance_record_example())),
    responses(
        (status = 200, description = "Performance record added successfully"),
        (status = 400, description = "Invalid request data"),
    )
)]
pub async fn post_performance_record(
    State(repository): State<Arc<Mutex<dyn Storage>>>,
    Json(performance_record): Json<PerformanceRecord>,
) -> Result<Json<PerformanceRecord>, (StatusCode, String)> {
    let namespace = "leaderboard";
    let performance_record = add_performance_record(namespace, performance_record, repository)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(Json(performance_record))
}

#[utoipa::path(
    post,
    operation_id = "post_challenge_performance_record_v1",
    tag = "leaderboard_v1",
    path = "/performance-record/{challenge_id}",
    context_path = "/api/v1",
    request_body(content = PerformanceRecord, example = json!(performance_record_example())),
    responses(
        (status = 200, description = "Performance record added successfully"),
        (status = 400, description = "Invalid request data"),
    )
)]
pub async fn post_challenge_performance_record(
    Path(challenge_id): Path<String>,
    State(repository): State<Arc<Mutex<dyn Storage>>>,
    Json(performance_record): Json<PerformanceRecord>,
) -> Result<Json<PerformanceRecord>, (StatusCode, String)> {
    let namespace = challenge_id.as_str();
    let performance_record = add_performance_record(namespace, performance_record, repository)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(Json(performance_record))
}
