use crate::storage::{RepositoryError, Storage};
use konnektoren_core::challenges::PerformanceRecord;
use std::sync::Arc;
use tokio::sync::Mutex;

const PERFORMANCE_RECORDS_LIMIT: usize = 10;

pub async fn add_performance_record(
    performance_record: PerformanceRecord,
    repository: Arc<Mutex<dyn Storage>>,
) -> Result<PerformanceRecord, RepositoryError> {
    let leaderboard = repository.lock().await.fetch_performance_records().await?;
    if leaderboard.len() > PERFORMANCE_RECORDS_LIMIT {
        // remove record with the lowest performance
        let record_to_remove = leaderboard
            .iter()
            .min_by_key(|record| record.performance_percentage)
            .unwrap();

        if record_to_remove.performance_percentage < performance_record.performance_percentage {
            repository
                .lock()
                .await
                .remove_performance_record(record_to_remove.clone())
                .await?;
            repository
                .lock()
                .await
                .add_performance_record(performance_record.clone())
                .await?;
            return Ok(performance_record);
        }
    } else {
        repository
            .lock()
            .await
            .add_performance_record(performance_record.clone())
            .await?;
    }

    Ok(performance_record)
}

pub async fn fetch_all_performance_records(
    repository: Arc<Mutex<dyn Storage>>,
) -> Result<Vec<PerformanceRecord>, RepositoryError> {
    let performance_records = repository.lock().await.fetch_performance_records().await?;
    Ok(performance_records)
}
