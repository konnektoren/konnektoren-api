use crate::storage::{CouponRepository, RepositoryError, Storage};
use konnektoren_core::prelude::{Coupon, CouponRedemptionError};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Debug, Error)]
pub enum CouponError {
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Coupon redemption error: {0}")]
    Redemption(#[from] CouponRedemptionError),
}

pub async fn create_coupon(
    coupon: Coupon,
    repository: Arc<Mutex<dyn Storage>>,
) -> Result<Coupon, CouponError> {
    let mut storage = repository.lock().await;
    CouponRepository::save(&mut *storage, coupon)
        .await
        .map_err(CouponError::Repository)
}

pub async fn get_coupon(
    code: String,
    repository: Arc<Mutex<dyn Storage>>,
) -> Result<Option<Coupon>, CouponError> {
    let storage = repository.lock().await;
    CouponRepository::fetch(&*storage, &code)
        .await
        .map_err(CouponError::Repository)
}

pub async fn list_coupons(repository: Arc<Mutex<dyn Storage>>) -> Result<Vec<Coupon>, CouponError> {
    let storage = repository.lock().await;
    CouponRepository::fetch_all(&*storage)
        .await
        .map_err(CouponError::Repository)
}

pub async fn validate_coupon(
    code: String,
    challenge_id: String,
    repository: Arc<Mutex<dyn Storage>>,
) -> Result<bool, CouponError> {
    let storage = repository.lock().await;
    let coupon = CouponRepository::fetch(&*storage, &code)
        .await
        .map_err(CouponError::Repository)?;

    match coupon {
        Some(coupon) => Ok(coupon.challenge_ids.contains(&challenge_id)
            && coupon.expiration_date > chrono::Utc::now()
            && coupon.uses_remaining > 0),
        None => Ok(false),
    }
}

pub async fn redeem_coupon(
    code: String,
    challenge_id: String,
    repository: Arc<Mutex<dyn Storage>>,
) -> Result<bool, CouponError> {
    let mut storage = repository.lock().await;

    if let Some(mut coupon) = CouponRepository::fetch(&*storage, &code)
        .await
        .map_err(CouponError::Repository)?
    {
        if !coupon.challenge_ids.contains(&challenge_id)
            || coupon.expiration_date < chrono::Utc::now()
            || coupon.uses_remaining == 0
        {
            return Ok(false);
        }

        let trace_id = "trace-id".to_string(); // TODO: implement proper tracing
        coupon.redeem(trace_id).map_err(CouponError::Redemption)?;
        CouponRepository::save(&mut *storage, coupon)
            .await
            .map_err(CouponError::Repository)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryRepository;
    use chrono::{Duration, Utc};

    #[tokio::test]
    async fn test_create_coupon() {
        let repository = Arc::new(Mutex::new(MemoryRepository::new()));
        let coupon = Coupon::new(
            "TEST123".to_string(),
            vec!["challenge1".to_string()],
            1,
            Utc::now() + Duration::days(7),
        );

        let result = create_coupon(coupon.clone(), repository).await;

        assert!(result.is_ok());
        let saved_coupon = result.unwrap();
        assert_eq!(saved_coupon.code, "TEST123");
    }

    #[tokio::test]
    async fn test_redeem_expired_coupon() {
        let repository = Arc::new(Mutex::new(MemoryRepository::new()));
        let coupon = Coupon::new(
            "TEST123".to_string(),
            vec!["challenge1".to_string()],
            1,
            Utc::now() - Duration::days(1), // Expired coupon
        );

        CouponRepository::save(&mut *repository.lock().await, coupon)
            .await
            .unwrap();

        let result =
            redeem_coupon("TEST123".to_string(), "challenge1".to_string(), repository).await;

        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should return false for expired coupon
    }

    #[tokio::test]
    async fn test_get_coupon() {
        let repository = Arc::new(Mutex::new(MemoryRepository::new()));
        let coupon = Coupon::new(
            "TEST123".to_string(),
            vec!["challenge1".to_string()],
            1,
            Utc::now() + Duration::days(7),
        );

        CouponRepository::save(&mut *repository.lock().await, coupon.clone())
            .await
            .unwrap();

        // Test fetching existing coupon
        let result = get_coupon("TEST123".to_string(), repository.clone()).await;
        assert!(result.is_ok());
        let fetched = result.unwrap().unwrap();
        assert_eq!(fetched.code, "TEST123");

        // Test fetching non-existent coupon
        let result = get_coupon("NONEXISTENT".to_string(), repository).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_list_coupons() {
        let repository = Arc::new(Mutex::new(MemoryRepository::new()));

        // Create and save multiple coupons
        let coupon1 = Coupon::new(
            "TEST1".to_string(),
            vec!["challenge1".to_string()],
            1,
            Utc::now() + Duration::days(7),
        );
        let coupon2 = Coupon::new(
            "TEST2".to_string(),
            vec!["challenge2".to_string()],
            2,
            Utc::now() + Duration::days(7),
        );

        CouponRepository::save(&mut *repository.lock().await, coupon1)
            .await
            .unwrap();
        CouponRepository::save(&mut *repository.lock().await, coupon2)
            .await
            .unwrap();

        let result = list_coupons(repository).await;
        assert!(result.is_ok());
        let coupons = result.unwrap();
        assert_eq!(coupons.len(), 2);
        assert!(coupons.iter().any(|c| c.code == "TEST1"));
        assert!(coupons.iter().any(|c| c.code == "TEST2"));
    }

    #[tokio::test]
    async fn test_validate_coupon() {
        let repository = Arc::new(Mutex::new(MemoryRepository::new()));
        let coupon = Coupon::new(
            "TEST123".to_string(),
            vec!["challenge1".to_string()],
            1,
            Utc::now() + Duration::days(7),
        );

        CouponRepository::save(&mut *repository.lock().await, coupon)
            .await
            .unwrap();

        // Test valid coupon and challenge
        let result = validate_coupon(
            "TEST123".to_string(),
            "challenge1".to_string(),
            repository.clone(),
        )
        .await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test invalid challenge
        let result = validate_coupon(
            "TEST123".to_string(),
            "invalid_challenge".to_string(),
            repository.clone(),
        )
        .await;
        assert!(result.is_ok());
        assert!(!result.unwrap());

        // Test non-existent coupon
        let result = validate_coupon(
            "NONEXISTENT".to_string(),
            "challenge1".to_string(),
            repository,
        )
        .await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_redeem_coupon_success() {
        let repository = Arc::new(Mutex::new(MemoryRepository::new()));
        let coupon = Coupon::new(
            "TEST123".to_string(),
            vec!["challenge1".to_string()],
            2, // Allow multiple uses
            Utc::now() + Duration::days(7),
        );

        CouponRepository::save(&mut *repository.lock().await, coupon)
            .await
            .unwrap();

        // First redemption should succeed
        let result = redeem_coupon(
            "TEST123".to_string(),
            "challenge1".to_string(),
            repository.clone(),
        )
        .await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Second redemption should fail due to trace ID
        let result =
            redeem_coupon("TEST123".to_string(), "challenge1".to_string(), repository).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_redeem_coupon_invalid_challenge() {
        let repository = Arc::new(Mutex::new(MemoryRepository::new()));
        let coupon = Coupon::new(
            "TEST123".to_string(),
            vec!["challenge1".to_string()],
            1,
            Utc::now() + Duration::days(7),
        );

        CouponRepository::save(&mut *repository.lock().await, coupon)
            .await
            .unwrap();

        let result = redeem_coupon(
            "TEST123".to_string(),
            "invalid_challenge".to_string(),
            repository,
        )
        .await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_redeem_nonexistent_coupon() {
        let repository = Arc::new(Mutex::new(MemoryRepository::new()));

        let result = redeem_coupon(
            "NONEXISTENT".to_string(),
            "challenge1".to_string(),
            repository,
        )
        .await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
