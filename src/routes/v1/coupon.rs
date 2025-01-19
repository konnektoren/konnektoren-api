use crate::services::v1::coupon::{self, CouponError};
use crate::storage::Storage;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use konnektoren_core::prelude::{Coupon, CouponRedemptionError};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CouponResponse {
    #[schema()]
    pub coupon: Option<Coupon>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CouponsResponse {
    #[schema()]
    pub coupons: Vec<Coupon>,
}

pub type CouponRequest = Coupon;

fn coupon_example() -> Coupon {
    Coupon::new(
        "EXAMPLE123".to_string(),
        vec!["challenge1".to_string()],
        1,
        chrono::Utc::now() + chrono::Duration::days(7),
    )
}

#[utoipa::path(
    post,
    operation_id = "create_coupon_v1",
    tag = "coupon_v1",
    path = "/coupons",
    context_path = "/api/v1",
    request_body(content = CouponRequest, example = json!(coupon_example())),
    responses(
        (status = 201, description = "Coupon created successfully", body = CouponResponse),
        (status = 409, description = "Coupon already exists"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn create_handler(
    State(repository): State<Arc<Mutex<dyn Storage>>>,
    Json(coupon): Json<Coupon>,
) -> Result<(StatusCode, Json<CouponResponse>), (StatusCode, String)> {
    match coupon::create_coupon(coupon, repository).await {
        Ok(saved_coupon) => Ok((
            StatusCode::CREATED,
            Json(CouponResponse {
                coupon: Some(saved_coupon),
            }),
        )),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    }
}

#[utoipa::path(
    get,
    operation_id = "get_coupon_v1",
    tag = "coupon_v1",
    path = "/coupons/{code}",
    params(
        ("code", description = "Coupon code to retrieve"),
    ),
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Coupon found", body = CouponResponse),
        (status = 404, description = "Coupon not found"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn get_handler(
    Path(code): Path<String>,
    State(repository): State<Arc<Mutex<dyn Storage>>>,
) -> Result<Json<CouponResponse>, (StatusCode, String)> {
    match coupon::get_coupon(code, repository).await {
        Ok(Some(coupon)) => Ok(Json(CouponResponse {
            coupon: Some(coupon),
        })),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Coupon not found".to_string())),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    }
}

#[utoipa::path(
    get,
    operation_id = "list_coupons_v1",
    tag = "coupon_v1",
    path = "/coupons",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "List of all coupons", body = CouponsResponse),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn list_handler(
    State(repository): State<Arc<Mutex<dyn Storage>>>,
) -> Result<Json<CouponsResponse>, (StatusCode, String)> {
    match coupon::list_coupons(repository).await {
        Ok(coupons) => Ok(Json(CouponsResponse { coupons })),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    }
}

#[utoipa::path(
    get,
    operation_id = "validate_coupon_v1",
    tag = "coupon_v1",
    path = "/coupons/{code}/validate/{challenge_id}",
    params(
        ("code", description = "Coupon code to validate"),
        ("challenge_id", description = "Challenge ID to validate against"),
    ),
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Coupon is valid"),
        (status = 403, description = "Coupon not valid for this challenge"),
        (status = 404, description = "Coupon not found"),
        (status = 410, description = "Coupon expired or no uses remaining"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn validate_handler(
    Path((code, challenge_id)): Path<(String, String)>,
    State(repository): State<Arc<Mutex<dyn Storage>>>,
) -> Result<StatusCode, (StatusCode, String)> {
    match coupon::validate_coupon(code, challenge_id, repository).await {
        Ok(true) => Ok(StatusCode::OK),
        Ok(false) => Err((
            StatusCode::FORBIDDEN,
            "Invalid or expired coupon".to_string(),
        )),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    }
}

#[utoipa::path(
    post,
    operation_id = "redeem_coupon_v1",
    tag = "coupon_v1",
    path = "/coupons/{code}/redeem/{challenge_id}",
    params(
        ("code", description = "Coupon code to redeem"),
        ("challenge_id", description = "Challenge ID to redeem for"),
    ),
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Coupon redeemed successfully"),
        (status = 403, description = "Coupon not valid for this challenge"),
        (status = 404, description = "Coupon not found"),
        (status = 409, description = "Failed to redeem coupon"),
        (status = 410, description = "Coupon expired or no uses remaining"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn redeem_handler(
    Path((code, challenge_id)): Path<(String, String)>,
    State(repository): State<Arc<Mutex<dyn Storage>>>,
) -> Result<StatusCode, (StatusCode, String)> {
    match coupon::redeem_coupon(code, challenge_id, repository).await {
        Ok(true) => Ok(StatusCode::OK),
        Ok(false) => Err((StatusCode::GONE, "Coupon cannot be redeemed".to_string())),
        Err(CouponError::Repository(err)) => {
            Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
        }
        Err(CouponError::Redemption(err)) => {
            let (status, message) = match err {
                CouponRedemptionError::Expired(date) => {
                    (StatusCode::GONE, format!("Coupon expired on {}", date))
                }
                CouponRedemptionError::AlreadyUsed => {
                    (StatusCode::CONFLICT, "Coupon already used".to_string())
                }
                CouponRedemptionError::InvalidChallenge => {
                    (StatusCode::FORBIDDEN, "Invalid challenge ID".to_string())
                }
                CouponRedemptionError::NoUsesRemaining => {
                    (StatusCode::GONE, "No uses remaining".to_string())
                }
            };
            Err((status, message))
        }
    }
}
