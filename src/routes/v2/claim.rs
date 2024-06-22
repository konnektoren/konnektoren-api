use axum::extract::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::services::v2::claim::claim_tokens_service;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClaimV2Request {
    #[schema(example = 123456)]
    pub id: i64,
    #[schema(example = "example_user")]
    pub user: String,
    #[serde(rename = "type")]
    #[schema(example = "claim")]
    pub request_type: String,
    #[schema(example = "0QB-_k5Rule-nKr6HWPIlkDyHb1xhDdbI77q7uwAFqmUmKjP")]
    pub address: String,
    #[schema(example = 1)]
    pub amount: u32,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ClaimV2Response {
    #[schema(example = true)]
    pub success: bool,
    #[schema(example = "0x1234567890")]
    pub raw_transaction: String,
    #[schema(example = "0QB-_k5Rule-nKr6HWPIlkDyHb1xhDdbI77q7uwAFqmUmKjP")]
    pub destination: String,
}

#[utoipa::path(
    post,
    operation_id = "claim_v2",
    tag = "claim_v2",
    path = "/claim",
    context_path = "/api/v2",
    request_body = ClaimV2Request,
    responses(
        (status = 200, description = "Token claimed successfully", body = ClaimV2Response),
        (status = 400, description = "Invalid request data"),
    )
)]
pub async fn claim_tokens(
    Json(payload): Json<ClaimV2Request>,
) -> Result<Json<ClaimV2Response>, (axum::http::StatusCode, String)> {
    claim_tokens_service(payload).await
}
