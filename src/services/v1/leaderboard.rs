use crate::storage::{RepositoryError, Storage};
use konnektoren_core::challenges::PerformanceRecord;
use std::sync::Arc;
use tokio::sync::Mutex;

const PERFORMANCE_RECORDS_LIMIT: usize = 10;

pub async fn add_performance_record(
    namespace: &str,
    performance_record: PerformanceRecord,
    repository: Arc<Mutex<dyn Storage>>,
) -> Result<PerformanceRecord, RepositoryError> {
    let mut storage = repository.lock().await;

    // First try to add directly
    match storage
        .add_performance_record(namespace, performance_record.clone())
        .await
    {
        Ok(record) => return Ok(record),
        Err(RepositoryError::LimitReached(_)) => {
            // If limit reached, get all records to compare
            let mut leaderboard = storage.fetch_performance_records(namespace).await?;
            leaderboard.sort();

            // If there are records and the new record is better than the worst one
            if let Some(worst_record) = leaderboard.last() {
                if worst_record > &performance_record {
                    // Remove the worst record
                    storage
                        .remove_performance_record(namespace, worst_record.clone())
                        .await?;

                    // Try adding the new record again
                    storage
                        .add_performance_record(namespace, performance_record.clone())
                        .await?;

                    return Ok(performance_record);
                }
            }
            Err(RepositoryError::LimitReached(PERFORMANCE_RECORDS_LIMIT))
        }
        Err(e) => Err(e),
    }
}

pub async fn fetch_all_performance_records(
    namespace: &str,
    repository: Arc<Mutex<dyn Storage>>,
) -> Result<Vec<PerformanceRecord>, RepositoryError> {
    let performance_records = repository
        .lock()
        .await
        .fetch_performance_records(namespace)
        .await?;
    Ok(performance_records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{LeaderboardRepository, MemoryRepository};
    use chrono::DateTime;

    #[test]
    fn test_sort_performance() {
        let a = PerformanceRecord {
            game_path_id: "".to_string(),
            profile_name: "a".to_string(),
            challenges_performance: vec![("".to_string(), 100, 100)],
            date: DateTime::from(DateTime::parse_from_rfc3339("2021-08-01T00:00:00Z").unwrap()),
            performance_percentage: 100,
            total_challenges: 0,
        };

        let b = PerformanceRecord {
            game_path_id: "".to_string(),
            profile_name: "b".to_string(),
            challenges_performance: vec![("".to_string(), 100, 400)],
            date: DateTime::from(DateTime::parse_from_rfc3339("2021-08-01T00:00:00Z").unwrap()),
            performance_percentage: 100,
            total_challenges: 0,
        };

        let c = PerformanceRecord {
            game_path_id: "".to_string(),
            profile_name: "c".to_string(),
            challenges_performance: vec![("".to_string(), 100, 300)],
            date: DateTime::from(DateTime::parse_from_rfc3339("2021-08-02T00:00:00Z").unwrap()),
            performance_percentage: 100,
            total_challenges: 0,
        };

        let d = PerformanceRecord {
            game_path_id: "".to_string(),
            profile_name: "d".to_string(),
            challenges_performance: vec![("".to_string(), 100, 200)],
            date: DateTime::from(DateTime::parse_from_rfc3339("2021-08-02T00:00:00Z").unwrap()),
            performance_percentage: 100,
            total_challenges: 0,
        };

        let mut leaderboard = vec![a.clone(), b.clone(), c.clone(), d.clone()];
        leaderboard.sort();

        assert_eq!(leaderboard, vec![a, d, c, b.clone()]);
        assert_eq!(leaderboard.last().unwrap(), &b);
    }

    #[test]
    fn test_sort_performance_and_date() {
        let a = PerformanceRecord {
            game_path_id: "".to_string(),
            profile_name: "a".to_string(),
            challenges_performance: vec![],
            date: DateTime::from(DateTime::parse_from_rfc3339("2021-08-01T00:00:00Z").unwrap()),
            performance_percentage: 100,
            total_challenges: 0,
        };

        let b = PerformanceRecord {
            game_path_id: "".to_string(),
            profile_name: "b".to_string(),
            challenges_performance: vec![],
            date: DateTime::from(DateTime::parse_from_rfc3339("2021-08-01T00:00:00Z").unwrap()),
            performance_percentage: 90,
            total_challenges: 0,
        };

        let c = PerformanceRecord {
            game_path_id: "".to_string(),
            profile_name: "c".to_string(),
            challenges_performance: vec![],
            date: DateTime::from(DateTime::parse_from_rfc3339("2021-08-02T00:00:00Z").unwrap()),
            performance_percentage: 100,
            total_challenges: 0,
        };

        let d = PerformanceRecord {
            game_path_id: "".to_string(),
            profile_name: "d".to_string(),
            challenges_performance: vec![],
            date: DateTime::from(DateTime::parse_from_rfc3339("2021-08-02T00:00:00Z").unwrap()),
            performance_percentage: 90,
            total_challenges: 0,
        };

        let mut vec = vec![a.clone(), b.clone(), c.clone(), d.clone()];
        vec.sort();
        assert_eq!(vec, vec![c, a, d, b]);
    }

    #[tokio::test]
    async fn test_add_limit_reached() {
        let storage = MemoryRepository::new();
        let repository = Arc::new(Mutex::new(storage));

        let namespace = "test";
        let mut records = vec![];

        // Fill up the leaderboard
        for i in 0..PERFORMANCE_RECORDS_LIMIT {
            let record = PerformanceRecord {
                profile_name: i.to_string(),
                challenges_performance: vec![("".to_string(), 100, 200)],
                date: DateTime::from(DateTime::parse_from_rfc3339("2021-08-01T00:00:00Z").unwrap()),
                performance_percentage: 100,
                total_challenges: 1,
                ..Default::default()
            };
            records.push(record.clone());

            // Use repository.lock() to access storage methods
            repository
                .lock()
                .await
                .add_performance_record(namespace, record)
                .await
                .unwrap();
        }

        let new_record = PerformanceRecord {
            profile_name: "new".to_string(),
            challenges_performance: vec![("".to_string(), 100, 100)],
            date: DateTime::from(DateTime::parse_from_rfc3339("2021-08-01T00:00:00Z").unwrap()),
            performance_percentage: 100,
            total_challenges: 1,
            ..Default::default()
        };

        let result = add_performance_record(namespace, new_record, repository.clone()).await;
        assert!(result.is_ok());

        let new_worse_record = PerformanceRecord {
            profile_name: "new_worse".to_string(),
            challenges_performance: vec![("".to_string(), 100, 500)],
            date: DateTime::from(DateTime::parse_from_rfc3339("2021-08-01T00:00:00Z").unwrap()),
            performance_percentage: 0,
            total_challenges: 1,
            ..Default::default()
        };

        let result = add_performance_record(namespace, new_worse_record, repository.clone()).await;
        assert!(matches!(result, Err(RepositoryError::LimitReached(_))));

        let new_best_record = PerformanceRecord {
            profile_name: "new_best".to_string(),
            challenges_performance: vec![("".to_string(), 100, 10)],
            date: DateTime::from(DateTime::parse_from_rfc3339("2021-08-01T00:00:00Z").unwrap()),
            performance_percentage: 100,
            total_challenges: 1,
            ..Default::default()
        };

        let result = add_performance_record(namespace, new_best_record, repository).await;
        assert!(result.is_ok());
    }
}
