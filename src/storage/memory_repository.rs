use crate::storage::{
    LeaderboardRepository, ProfileRepository, RepositoryError, ReviewRepository, Storage,
    WindowedCounterRepository,
};
use async_trait::async_trait;
use konnektoren_core::challenges::{PerformanceRecord, Review};
use konnektoren_core::prelude::PlayerProfile;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use yew_chat::prelude::{Message, MessageReceiver, MessageSender, ReceiveError, SendError};
#[cfg(feature = "chat")]
use yew_chat::server::MemoryMessageStorage;
use yew_chat::server::MessageStorage;

const PERFORMANCE_RECORDS_LIMIT: usize = 10;

pub struct MemoryRepository {
    profiles: HashMap<String, PlayerProfile>,
    performance_records: Vec<PerformanceRecord>,
    reviews: HashMap<String, Vec<Review>>,
    #[cfg(feature = "chat")]
    message_storage: MemoryMessageStorage,
    active_users: HashMap<String, Vec<u64>>,
}

impl MemoryRepository {
    pub fn new() -> Self {
        MemoryRepository {
            profiles: HashMap::new(),
            performance_records: Vec::new(),
            reviews: HashMap::new(),
            #[cfg(feature = "chat")]
            message_storage: MemoryMessageStorage::new(),
            active_users: HashMap::new(),
        }
    }

    fn cleanup_old_entries(&mut self, namespace: &str) {
        if let Some(timestamps) = self.active_users.get_mut(namespace) {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let window = 24 * 60 * 60;
            timestamps.retain(|&timestamp| current_time - timestamp < window);
        }
    }
}

impl Storage for MemoryRepository {}

#[async_trait]
impl ProfileRepository for MemoryRepository {
    async fn fetch(&self, profile_id: String) -> Result<PlayerProfile, RepositoryError> {
        match self.profiles.get(&profile_id) {
            Some(profile) => Ok(profile.clone()),
            None => Err(RepositoryError::NotFound(profile_id.clone())),
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
            None => Err(RepositoryError::NotFound(
                performance_record.game_path_id.clone(),
            )),
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
            .ok_or(RepositoryError::NotFound(namespace.to_string()))?;
        let total: u32 = reviews.iter().map(|review| review.rating as u32).sum();
        let count = reviews.len() as f64;

        Ok(if count > 0.0 {
            total as f64 / count
        } else {
            0.0
        })
    }

    async fn fetch_all_reviews(&self) -> Result<Vec<Review>, RepositoryError> {
        Ok(self.reviews.values().flatten().cloned().collect())
    }
}

#[cfg(feature = "chat")]
#[async_trait]
impl MessageReceiver for MemoryRepository {
    async fn receive_messages(&self, channel: &str) -> Result<Vec<Message>, ReceiveError> {
        self.message_storage.receive_messages(channel).await
    }
}

#[cfg(feature = "chat")]
#[async_trait]
impl MessageSender for MemoryRepository {
    async fn send_message(&self, channel: &str, message: Message) -> Result<(), SendError> {
        self.message_storage.send_message(channel, message).await
    }
}

#[cfg(feature = "chat")]
impl MessageStorage for MemoryRepository {}

#[async_trait]
impl WindowedCounterRepository for MemoryRepository {
    async fn get_active_count(&self, namespace: &str) -> Result<u32, RepositoryError> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let window = 24 * 60 * 60; // 24 hours in seconds

        let count = self
            .active_users
            .get(namespace)
            .map(|timestamps| {
                timestamps
                    .iter()
                    .filter(|&&timestamp| current_time - timestamp < window)
                    .count() as u32
            })
            .unwrap_or(0);

        Ok(count)
    }

    async fn record_presence(&mut self, namespace: &str) -> Result<u32, RepositoryError> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Clean up old entries first
        self.cleanup_old_entries(namespace);

        // Record new presence
        self.active_users
            .entry(namespace.to_string())
            .or_insert_with(Vec::new)
            .push(current_time);

        self.get_active_count(namespace).await
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

    #[tokio::test]
    async fn test_windowed_counter() {
        let mut repo = MemoryRepository::new();
        let namespace = "test_challenge";

        // Test initial count
        let count = repo.get_active_count(namespace).await.unwrap();
        assert_eq!(count, 0);

        // Test recording presence
        let count = repo.record_presence(namespace).await.unwrap();
        assert_eq!(count, 1);

        // Test multiple records
        repo.record_presence(namespace).await.unwrap();
        let count = repo.record_presence(namespace).await.unwrap();
        assert_eq!(count, 3);

        // Test different namespace
        let count = repo.get_active_count("other_namespace").await.unwrap();
        assert_eq!(count, 0);
    }
}
