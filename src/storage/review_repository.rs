use crate::storage::error::RepositoryError;
use async_trait::async_trait;
use konnektoren_core::challenges::Review;

#[async_trait]
pub trait ReviewRepository: Send + Sync {
    async fn store_review(&mut self, review: Review) -> Result<(), RepositoryError>;
    async fn fetch_reviews(&self, namespace: &str) -> Result<Vec<Review>, RepositoryError>;
    async fn fetch_average_rating(&self, namespace: &str) -> Result<f64, RepositoryError>;
}
