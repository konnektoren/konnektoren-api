use crate::storage::ProfileRepository;
use anyhow::Error;
use konnektoren_core::prelude::PlayerProfile;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn fetch_profile(
    profile_id: String,
    repository: Arc<Mutex<dyn ProfileRepository>>,
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

pub async fn save_profile(
    profile: PlayerProfile,
    repository: Arc<Mutex<dyn ProfileRepository>>,
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

    use crate::storage::RepositoryError;
    use async_trait::async_trait;
    use mockall::predicate::*;
    use mockall::*;

    mock! {
        ProfileRepository {}

        #[async_trait]
        impl ProfileRepository for ProfileRepository {
            async fn fetch(&self, profile_id: String) -> Result<PlayerProfile, RepositoryError>;
            async fn save(&mut self, profile: PlayerProfile) -> Result<PlayerProfile, RepositoryError>;
        }
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
            .returning(|profile| Ok(profile));
        let profile = PlayerProfile::new("example_user_id".to_string());
        let saved_profile = save_profile(profile, Arc::new(Mutex::new(mock)))
            .await
            .unwrap();
        assert_eq!(saved_profile.id, "example_user_id");
    }
}
