use crate::storage::error::RepositoryError;
use async_trait::async_trait;
use konnektoren_core::challenges::PerformanceRecord;

#[async_trait]
pub trait LeaderboardRepository: Send + Sync {
    async fn fetch_performance_records(
        &self,
        namespace: &str,
    ) -> Result<Vec<PerformanceRecord>, RepositoryError>;
    async fn add_performance_record(
        &mut self,
        namespace: &str,
        performance_record: PerformanceRecord,
    ) -> Result<PerformanceRecord, RepositoryError>;

    async fn remove_performance_record(
        &mut self,
        namespace: &str,
        performance_record: PerformanceRecord,
    ) -> Result<PerformanceRecord, RepositoryError>;
}
