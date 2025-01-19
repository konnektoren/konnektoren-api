use crate::storage::{ProfileRepository, Storage};
use anyhow::Error;
use konnektoren_core::prelude::PlayerProfile;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn fetch_profile(
    profile_id: String,
    repository: Arc<Mutex<dyn Storage>>,
) -> Result<PlayerProfile, Error> {
    let mut storage = repository.lock().await;
    let profile = ProfileRepository::fetch(&*storage, profile_id)
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
    let mut storage = repository.lock().await;
    let profiles = ProfileRepository::fetch_all(&*storage)
        .await
        .map_err(|err| {
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
    let mut storage = repository.lock().await;
    log::info!("Received profile: {:?}", profile);
    let saved_profile = ProfileRepository::save(&mut *storage, profile)
        .await
        .map_err(|err| {
            log::error!("Error saving profile: {:?}", err);
            err
        })?;
    log::info!("Saved profile: {:?}", saved_profile);
    Ok(saved_profile)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryRepository;
    use konnektoren_core::prelude::PlayerProfile;

    #[tokio::test]
    async fn test_fetch_profile() {
        let mut repository = MemoryRepository::new();
        let profile = PlayerProfile::new("example_user_id".to_string());

        // Save the profile first
        repository.save(profile.clone()).await.unwrap();

        let fetched_profile = fetch_profile(
            "example_user_id".to_string(),
            Arc::new(Mutex::new(repository)),
        )
        .await
        .unwrap();

        assert_eq!(fetched_profile.id, "example_user_id");
    }

    #[tokio::test]
    async fn test_fetch_all_profiles() {
        let mut repository = MemoryRepository::new();

        // Save multiple profiles
        repository
            .save(PlayerProfile::new("user1".to_string()))
            .await
            .unwrap();
        repository
            .save(PlayerProfile::new("user2".to_string()))
            .await
            .unwrap();

        let profiles = fetch_all_profiles(Arc::new(Mutex::new(repository)))
            .await
            .unwrap();

        assert_eq!(profiles.len(), 2);
        assert!(profiles.iter().any(|p| p.id == "user1"));
        assert!(profiles.iter().any(|p| p.id == "user2"));
    }

    #[tokio::test]
    async fn test_save_profile() {
        let repository = MemoryRepository::new();
        let profile = PlayerProfile::new("example_user_id".to_string());

        let saved_profile = save_profile(profile.clone(), Arc::new(Mutex::new(repository)))
            .await
            .unwrap();

        assert_eq!(saved_profile.id, "example_user_id");
    }

    #[tokio::test]
    async fn test_fetch_profile_error() {
        let repository = MemoryRepository::new();

        let result = fetch_profile(
            "nonexistent_id".to_string(),
            Arc::new(Mutex::new(repository)),
        )
        .await;

        assert!(result.is_err());
    }
}
