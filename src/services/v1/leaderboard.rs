use crate::storage::{RepositoryError, Storage};
use chrono::Duration;
use konnektoren_core::challenges::{PerformanceRecord, Timed};
use std::sync::Arc;
use tokio::sync::Mutex;

const PERFORMANCE_RECORDS_LIMIT: usize = 10;

pub async fn add_performance_record(
    namespace: &str,
    performance_record: PerformanceRecord,
    repository: Arc<Mutex<dyn Storage>>,
) -> Result<PerformanceRecord, RepositoryError> {
    let mut leaderboard = repository
        .lock()
        .await
        .fetch_performance_records(namespace)
        .await?;

    if leaderboard.len() >= PERFORMANCE_RECORDS_LIMIT {
        // remove record with the lowest performance
        leaderboard.sort();

        let record_to_remove = leaderboard.last().unwrap();

        if record_to_remove.performance_percentage <= performance_record.performance_percentage {
            repository
                .lock()
                .await
                .remove_performance_record(namespace, record_to_remove.clone())
                .await?;
            repository
                .lock()
                .await
                .add_performance_record(namespace, performance_record.clone())
                .await?;
            return Ok(performance_record);
        }
    } else {
        repository
            .lock()
            .await
            .add_performance_record(namespace, performance_record.clone())
            .await?;
    }

    Ok(performance_record)
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
    use chrono::DateTime;
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

        assert_eq!(a.cmp(&b), std::cmp::Ordering::Less);
        assert_eq!(a.cmp(&c), std::cmp::Ordering::Greater);
        assert_eq!(a.cmp(&d), std::cmp::Ordering::Less);
        assert_eq!(b.cmp(&c), std::cmp::Ordering::Greater);
        assert_eq!(b.cmp(&d), std::cmp::Ordering::Greater);

        let mut vec = vec![a.clone(), b.clone(), c.clone(), d.clone()];
        vec.sort();
        assert_eq!(vec, vec![c, a, d, b]);
    }
}
