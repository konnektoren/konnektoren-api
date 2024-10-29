use crate::storage::error::RepositoryError;
use async_trait::async_trait;

#[async_trait]
pub trait WindowedCounterRepository: Send + Sync {
    async fn get_active_count(&self, namespace: &str) -> Result<u32, RepositoryError>;
    async fn record_presence(&mut self, namespace: &str) -> Result<u32, RepositoryError>;
}
