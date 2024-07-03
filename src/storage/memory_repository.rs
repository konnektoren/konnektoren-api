use crate::storage::{ProfileRepository, RepositoryError};
use async_trait::async_trait;
use konnektoren_core::prelude::PlayerProfile;
use std::collections::HashMap;

pub struct MemoryRepository {
    profiles: HashMap<String, PlayerProfile>,
}

impl MemoryRepository {
    pub fn new() -> Self {
        MemoryRepository {
            profiles: HashMap::new(),
        }
    }
}

#[async_trait]
impl ProfileRepository for MemoryRepository {
    async fn fetch(&self, profile_id: String) -> Result<PlayerProfile, RepositoryError> {
        match self.profiles.get(&profile_id) {
            Some(profile) => Ok(profile.clone()),
            None => Err(RepositoryError::NotFound),
        }
    }

    async fn save(&mut self, profile: PlayerProfile) -> Result<PlayerProfile, RepositoryError> {
        self.profiles.insert(profile.id.clone(), profile.clone());
        Ok(profile)
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
}
