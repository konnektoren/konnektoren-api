use crate::routes::v1::claim::ClaimRequest;
use axum::{http::StatusCode, Json};

pub async fn claim_tokens_service(
    payload: ClaimRequest,
) -> Result<Json<&'static str>, (StatusCode, String)> {
    if payload.request_type != "claim" {
        return Err((StatusCode::BAD_REQUEST, "Invalid request type".into()));
    }

    println!("Received claim request: {:?}", payload);

    Ok(Json("Token claimed successfully"))
}
