use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use utoipa::ToSchema;

use crate::storage::Storage;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ReadinessResponse {
    pub status: String,
    pub checks: HealthChecks,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct HealthChecks {
    pub database: String,
    #[cfg(feature = "redis")]
    pub redis: String,
}

/// Health check endpoint - always returns OK if the service is running
#[utoipa::path(
    get,
    operation_id = "health_check",
    tag = "health",
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
    )
)]
pub async fn health_check() -> Result<Json<HealthResponse>, StatusCode> {
    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
}

/// Readiness check endpoint - checks if the service is ready to serve traffic
#[utoipa::path(
    get,
    operation_id = "readiness_check",
    tag = "health",
    path = "/ready",
    responses(
        (status = 200, description = "Service is ready", body = ReadinessResponse),
        (status = 503, description = "Service is not ready"),
    )
)]
pub async fn readiness_check(
    State(repository): State<Arc<Mutex<dyn Storage>>>,
) -> Result<Json<ReadinessResponse>, (StatusCode, Json<ReadinessResponse>)> {
    let mut checks = HealthChecks {
        database: "unknown".to_string(),
        #[cfg(feature = "redis")]
        redis: "unknown".to_string(),
    };

    let mut all_healthy = true;

    // Check database/storage connectivity
    match test_storage_connection(&repository).await {
        Ok(_) => checks.database = "healthy".to_string(),
        Err(_) => {
            checks.database = "unhealthy".to_string();
            all_healthy = false;
        }
    }

    #[cfg(feature = "redis")]
    {
        match test_redis_connection(&repository).await {
            Ok(_) => checks.redis = "healthy".to_string(),
            Err(_) => {
                checks.redis = "unhealthy".to_string();
                all_healthy = false;
            }
        }
    }

    let response = ReadinessResponse {
        status: if all_healthy { "ready" } else { "not_ready" }.to_string(),
        checks,
    };

    if all_healthy {
        Ok(Json(response))
    } else {
        Err((StatusCode::SERVICE_UNAVAILABLE, Json(response)))
    }
}

async fn test_storage_connection(
    repository: &Arc<Mutex<dyn Storage>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use crate::storage::ProfileRepository;

    // Try to perform a simple operation to test the connection
    let storage = repository.lock().await;
    let _ = ProfileRepository::fetch_all(&*storage).await?;
    Ok(())
}

#[cfg(feature = "redis")]
async fn test_redis_connection(
    repository: &Arc<Mutex<dyn Storage>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // For Redis, we can test the connection by trying to get active count
    use crate::storage::WindowedCounterRepository;

    let storage = repository.lock().await;
    let _ = storage.get_active_count("health_check").await?;
    Ok(())
}
