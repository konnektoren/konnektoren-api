use crate::storage::{ReviewRepository, Storage};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use utoipa::ToSchema;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct Review {
    pub challenge_id: String,
    pub rating: u8,
    pub comment: Option<String>,
}

impl Into<konnektoren_core::challenges::Review> for Review {
    fn into(self) -> konnektoren_core::challenges::Review {
        konnektoren_core::challenges::Review {
            challenge_id: self.challenge_id,
            rating: self.rating,
            comment: self.comment,
        }
    }
}

impl From<konnektoren_core::challenges::Review> for Review {
    fn from(review: konnektoren_core::challenges::Review) -> Self {
        Review {
            challenge_id: review.challenge_id,
            rating: review.rating,
            comment: review.comment,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ReviewResponse {
    #[schema()]
    pub review: Option<Review>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ReviewsResponse {
    #[schema()]
    pub reviews: Vec<Review>,
}

fn review_example() -> Review {
    Review {
        challenge_id: "example_challenge_id".to_string(),
        rating: 5,
        comment: Some("Great challenge!".to_string()),
    }
}

#[utoipa::path(
    get,
    operation_id = "get_review",
    tag = "review",
    path = "/reviews/{challenge_id}",
    params(
        ("challenge_id", description = "Id for the challenge to retrieve reviews"),
    ),
    context_path = "/api/v1",
    responses(
    (status = 200, description = "Reviews loaded successfully", body = ReviewsResponse),
    (status = 400, description = "Invalid request data"),
    )
)]
pub async fn get_reviews(
    State(repository): State<Arc<Mutex<dyn Storage>>>,
    Path(challenge_id): Path<String>,
) -> Result<Json<ReviewsResponse>, (StatusCode, String)> {
    let reviews = crate::services::v1::review::fetch_reviews(challenge_id, repository)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let reviews: Vec<Review> = reviews.into_iter().map(|r| r.into()).collect();
    Ok(Json(ReviewsResponse { reviews }))
}

#[utoipa::path(
    get,
    operation_id = "get_average_rating",
    tag = "review",
    path = "/reviews/{challenge_id}/average",
    params(
        ("challenge_id", description = "Id for the challenge to calculate average rating"),
    ),
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Average rating calculated successfully", body = f64),
        (status = 404, description = "Challenge not found"),
    )
)]
pub async fn get_average_rating(
    State(repository): State<Arc<Mutex<dyn Storage>>>,
    Path(challenge_id): Path<String>,
) -> Result<Json<f64>, (StatusCode, String)> {
    let average_rating =
        crate::services::v1::review::fetch_average_rating(challenge_id, repository)
            .await
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(Json(average_rating))
}

#[utoipa::path(
    post,
    operation_id = "post_review",
    tag = "review",
    path = "/reviews",
    context_path = "/api/v1",
    request_body(content = Review, example = json!(review_example())),
    responses(
        (status = 200, description = "Review successfully saved"),
        (status = 400, description = "Invalid request data"),
    )
)]
pub async fn post_review(
    State(repository): State<Arc<Mutex<dyn Storage>>>,
    Json(review): Json<Review>,
) -> Result<Json<Review>, (StatusCode, String)> {
    crate::services::v1::review::store_review(review.clone().into(), repository)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(Json(review))
}

#[utoipa::path(
    get,
    operation_id = "get_all_reviews",
    tag = "review",
    path = "/reviews",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Reviews loaded successfully", body = ReviewsResponse),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn get_all_reviews(
    State(repository): State<Arc<Mutex<dyn Storage>>>,
) -> Result<Json<ReviewsResponse>, (StatusCode, String)> {
    let reviews = crate::services::v1::review::fetch_all_reviews(repository)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let reviews: Vec<Review> = reviews.into_iter().map(|r| r.into()).collect();
    Ok(Json(ReviewsResponse { reviews }))
}
