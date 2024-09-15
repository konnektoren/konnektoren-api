use crate::storage::{
    LeaderboardRepository, ProfileRepository, RepositoryError, ReviewRepository, Storage,
};
use async_trait::async_trait;
use konnektoren_core::challenges::{PerformanceRecord, Review};
use konnektoren_core::prelude::PlayerProfile;
use std::collections::HashMap;

const PERFORMANCE_RECORDS_LIMIT: usize = 10;

pub struct MemoryRepository {
    profiles: HashMap<String, PlayerProfile>,
    performance_records: Vec<PerformanceRecord>,
    reviews: HashMap<String, Vec<Review>>,
}

impl MemoryRepository {
    pub fn new() -> Self {
        MemoryRepository {
            profiles: HashMap::new(),
            performance_records: Vec::new(),
            reviews: HashMap::new(),
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
    async fn fetch_performance_records(
        &self,
        _namespace: &str,
    ) -> Result<Vec<PerformanceRecord>, RepositoryError> {
        Ok(self.performance_records.clone())
    }

    async fn add_performance_record(
        &mut self,
        _namespace: &str,
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
        _namespace: &str,
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

#[async_trait]
impl ReviewRepository for MemoryRepository {
    async fn store_review(&mut self, review: Review) -> Result<(), RepositoryError> {
        self.reviews
            .entry(review.challenge_id.clone())
            .or_insert_with(Vec::new)
            .push(review);
        Ok(())
    }

    async fn fetch_reviews(&self, namespace: &str) -> Result<Vec<Review>, RepositoryError> {
        Ok(self
            .reviews
            .values()
            .flatten()
            .filter(|review| review.challenge_id == *namespace)
            .cloned()
            .collect())
    }

    async fn fetch_average_rating(&self, namespace: &str) -> Result<f64, RepositoryError> {
        let reviews = self
            .reviews
            .get(namespace)
            .ok_or(RepositoryError::NotFound)?;
        let total: u32 = reviews.iter().map(|review| review.rating as u32).sum();
        let count = reviews.len() as f64;

        Ok(if count > 0.0 {
            total as f64 / count
        } else {
            0.0
        })
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

    #[tokio::test]
    async fn test_fetch_reviews() {
        let mut repo = MemoryRepository::new();
        let review1 = Review {
            challenge_id: "example_challenge_id".to_string(),
            rating: 5,
            comment: None,
        };
        let review2 = Review {
            challenge_id: "example_challenge_id".to_string(),
            rating: 3,
            comment: None,
        };
        repo.store_review(review1.clone()).await.unwrap();
        repo.store_review(review2.clone()).await.unwrap();
        let reviews = ReviewRepository::fetch_reviews(&repo, "example_challenge_id")
            .await
            .unwrap();
        assert_eq!(reviews.len(), 2);
        assert!(reviews.contains(&review1));
        assert!(reviews.contains(&review2));
    }

    #[tokio::test]
    async fn test_fetch_average_rating() {
        let mut repo = MemoryRepository::new();
        let review1 = Review {
            challenge_id: "example_challenge_id".to_string(),
            rating: 5,
            comment: None,
        };
        let review2 = Review {
            challenge_id: "example_challenge_id".to_string(),
            rating: 3,
            comment: None,
        };
        repo.store_review(review1.clone()).await.unwrap();
        repo.store_review(review2.clone()).await.unwrap();
        let average_rating = ReviewRepository::fetch_average_rating(&repo, "example_challenge_id")
            .await
            .unwrap();
        assert_eq!(average_rating, 4.0);
    }
}
