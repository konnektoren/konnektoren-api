use crate::storage::Storage;
use anyhow::Error;
use konnektoren_core::prelude::PlayerProfile;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn fetch_profile(
    profile_id: String,
    repository: Arc<Mutex<dyn Storage>>,
) -> Result<PlayerProfile, Error> {
    let profile = repository
        .lock()
        .await
        .fetch(profile_id)
        .await
        .map_err(|err| {
            log::error!("Error fetching profile: {:?}", err);
            err
        })?;
    log::info!("Returning profile: {:?}", profile);
    Ok(profile)
}

pub async fn fetch_all_profiles(
    repository: Arc<Mutex<dyn Storage>>,
) -> Result<Vec<PlayerProfile>, Error> {
    let profiles = repository.lock().await.fetch_all().await.map_err(|err| {
        log::error!("Error fetching profiles: {:?}", err);
        err
    })?;
    log::info!("Returning profiles: {:?}", profiles);
    Ok(profiles)
}

pub async fn save_profile(
    profile: PlayerProfile,
    repository: Arc<Mutex<dyn Storage>>,
) -> Result<PlayerProfile, Error> {
    log::info!("Received profile: {:?}", profile);
    let saved_profile = repository.lock().await.save(profile).await.map_err(|err| {
        log::error!("Error saving profile: {:?}", err);
        err
    })?;
    log::info!("Saved profile: {:?}", saved_profile);
    Ok(saved_profile)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{
        LeaderboardRepository, ProfileRepository, RepositoryError, ReviewRepository,
        WindowedCounterRepository,
    };
    use async_trait::async_trait;
    use konnektoren_core::challenges::{PerformanceRecord, Review};
    use konnektoren_core::prelude::PlayerProfile;
    use mockall::predicate::*;
    use mockall::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[cfg(feature = "chat")]
    use yew_chat::{
        prelude::{Message, MessageReceiver, MessageSender, ReceiveError, SendError},
        server::MessageStorage,
    };

    mock! {
        pub ProfileRepository {}

        #[async_trait]
        impl LeaderboardRepository for ProfileRepository {
            async fn fetch_performance_records(&self, namespace: &str) -> Result<Vec<PerformanceRecord>, RepositoryError>;
            async fn add_performance_record(&mut self, namespace: &str, performance_record: PerformanceRecord) -> Result<PerformanceRecord, RepositoryError>;
            async fn remove_performance_record(&mut self, namespace: &str, performance_record: PerformanceRecord) -> Result<PerformanceRecord, RepositoryError>;
        }

        #[async_trait]
        impl ProfileRepository for ProfileRepository {
            async fn fetch(&self, profile_id: String) -> Result<PlayerProfile, RepositoryError>;
            async fn fetch_all(&self) -> Result<Vec<PlayerProfile>, RepositoryError>;
            async fn save(&mut self, profile: PlayerProfile) -> Result<PlayerProfile, RepositoryError>;
        }

        #[async_trait]
        impl ReviewRepository for ProfileRepository {
            async fn store_review(&mut self, review: Review) -> Result<(), RepositoryError>;
            async fn fetch_reviews(&self, namespace: &str) -> Result<Vec<Review>, RepositoryError>;
            async fn fetch_all_reviews(&self) -> Result<Vec<Review>, RepositoryError>;
            async fn fetch_average_rating(&self, namespace: &str) -> Result<f64, RepositoryError>;
        }

        #[async_trait]
        impl WindowedCounterRepository for ProfileRepository {
            async fn get_active_count(&self, namespace: &str) -> Result<u32, RepositoryError>;
            async fn record_presence(&mut self, namespace: &str) -> Result<u32, RepositoryError>;
        }

        #[cfg(feature = "chat")]
        #[async_trait]
        impl MessageReceiver for ProfileRepository {
            async fn receive_messages(&self, channel: &str) -> Result<Vec<Message>, ReceiveError> {
                self.message_storage.receive_messages(channel).await
            }
        }

        #[cfg(feature = "chat")]
        #[async_trait]
        impl MessageSender for ProfileRepository {
            async fn send_message(&self, channel: &str, message: Message) -> Result<(), SendError> {
                self.message_storage.send_message(channel, message).await
            }
        }

        #[cfg(feature = "chat")]
        impl MessageStorage for ProfileRepository {}

        #[async_trait]
        impl Storage for ProfileRepository {}
    }

    #[tokio::test]
    async fn test_fetch_profile() {
        let mut mock = MockProfileRepository::new();
        mock.expect_fetch()
            .with(eq("example_user_id".to_string()))
            .times(1)
            .returning(|_| Ok(PlayerProfile::new("example_user_id".to_string())));

        let profile = fetch_profile("example_user_id".to_string(), Arc::new(Mutex::new(mock)))
            .await
            .unwrap();

        assert_eq!(profile.id, "example_user_id");
    }

    #[tokio::test]
    async fn test_save_profile() {
        let mut mock = MockProfileRepository::new();
        mock.expect_save()
            .withf(|profile| profile.id == "example_user_id")
            .times(1)
            .returning(|profile| Ok(profile.clone()));

        let profile = PlayerProfile::new("example_user_id".to_string());
        let saved_profile = save_profile(profile, Arc::new(Mutex::new(mock)))
            .await
            .unwrap();

        assert_eq!(saved_profile.id, "example_user_id");
    }
}
