use crate::storage::leaderboard_repository::LeaderboardRepository;
use crate::storage::{ProfileRepository, RepositoryError, Storage};
use async_trait::async_trait;
use konnektoren_core::challenges::PerformanceRecord;
use konnektoren_core::prelude::PlayerProfile;
use std::collections::HashMap;

const PERFORMANCE_RECORDS_LIMIT: usize = 10;

pub struct MemoryRepository {
    profiles: HashMap<String, PlayerProfile>,
    performance_records: Vec<PerformanceRecord>,
}

impl MemoryRepository {
    pub fn new() -> Self {
        MemoryRepository {
            profiles: HashMap::new(),
            performance_records: Vec::new(),
        }
    }
}

impl Storage for MemoryRepository {}

#[async_trait]
impl ProfileRepository for MemoryRepository {
    async fn fetch(&self, profile_id: String) -> Result<PlayerProfile, RepositoryError> {
        match self.profiles.get(&profile_id) {
            Some(profile) => Ok(profile.clone()),
            None => Err(RepositoryError::NotFound),
        }
    }

    async fn fetch_all(&self) -> Result<Vec<PlayerProfile>, RepositoryError> {
        Ok(self.profiles.values().cloned().collect())
    }

    async fn save(&mut self, profile: PlayerProfile) -> Result<PlayerProfile, RepositoryError> {
        self.profiles.insert(profile.id.clone(), profile.clone());
        Ok(profile)
    }
}

#[async_trait]
impl LeaderboardRepository for MemoryRepository {
    async fn fetch_performance_records(&self) -> Result<Vec<PerformanceRecord>, RepositoryError> {
        Ok(self.performance_records.clone())
    }

    async fn add_performance_record(
        &mut self,
        performance_record: PerformanceRecord,
    ) -> Result<PerformanceRecord, RepositoryError> {
        if self.performance_records.len() < PERFORMANCE_RECORDS_LIMIT {
            self.performance_records.push(performance_record.clone());
        } else {
            return Err(RepositoryError::LimitReached(PERFORMANCE_RECORDS_LIMIT));
        }
        Ok(performance_record)
    }

    async fn remove_performance_record(
        &mut self,
        performance_record: PerformanceRecord,
    ) -> Result<PerformanceRecord, RepositoryError> {
        let index = self
            .performance_records
            .iter()
            .position(|r| r == &performance_record);
        match index {
            Some(i) => {
                self.performance_records.remove(i);
                Ok(performance_record)
            }
            None => Err(RepositoryError::NotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_profile() {
        let mut repo = MemoryRepository::new();
        let profile = PlayerProfile::new("example_user_id".to_string());
        repo.save(profile.clone()).await.unwrap();
        let fetched_profile = repo.fetch("example_user_id".to_string()).await.unwrap();
        assert_eq!(profile, fetched_profile);
    }

    #[tokio::test]
    async fn test_fetch_all_profiles() {
        let mut repo = MemoryRepository::new();
        let profile1 = PlayerProfile::new("example_user_id1".to_string());
        let profile2 = PlayerProfile::new("example_user_id2".to_string());
        repo.save(profile1.clone()).await.unwrap();
        repo.save(profile2.clone()).await.unwrap();
        let profiles = ProfileRepository::fetch_all(&repo).await.unwrap();
        assert_eq!(profiles.len(), 2);
        assert!(profiles.contains(&profile1));
        assert!(profiles.contains(&profile2));
    }
}
