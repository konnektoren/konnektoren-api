use crate::storage::error::RepositoryError;
use async_trait::async_trait;
use konnektoren_core::marketplace::Coupon;

#[async_trait]
pub trait CouponRepository: Send + Sync {
    async fn fetch(&self, coupon_code: &str) -> Result<Option<Coupon>, RepositoryError>;
    async fn fetch_all(&self) -> Result<Vec<Coupon>, RepositoryError>;
    async fn save(&mut self, coupon: Coupon) -> Result<Coupon, RepositoryError>;
}
