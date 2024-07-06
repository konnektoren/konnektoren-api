use crate::storage::{ProfileRepository, RepositoryError};
use async_trait::async_trait;
use konnektoren_core::prelude::PlayerProfile;
pub struct RedisStorage {
    client: redis::Client,
}

const PROFILES_HASH: &str = "profiles";

impl RedisStorage {
    pub fn new(url: &str) -> Self {
        Self {
            client: redis::Client::open(url).expect("Invalid Redis URL"),
        }
    }
}

#[async_trait]
impl ProfileRepository for RedisStorage {
    async fn fetch(&self, profile_id: String) -> Result<PlayerProfile, RepositoryError> {
        let mut connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| RepositoryError::InternalError(err.to_string()))?;
        let profile_json: String = redis::cmd("HGET")
            .arg(PROFILES_HASH)
            .arg(&profile_id)
            .query_async(&mut connection)
            .await
            .map_err(|err| RepositoryError::InternalError(err.to_string()))?;
        let profile: PlayerProfile = serde_json::from_str(&profile_json)
            .map_err(|err| RepositoryError::InternalError(err.to_string()))?;
        Ok(profile)
    }

    async fn fetch_all(&self) -> Result<Vec<PlayerProfile>, RepositoryError> {
        let mut connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| RepositoryError::InternalError(err.to_string()))?;
        let profiles_data: Vec<String> = redis::cmd("HVALS")
            .arg(PROFILES_HASH)
            .query_async(&mut connection)
            .await
            .map_err(|err| RepositoryError::InternalError(err.to_string()))?;
        let mut profiles = Vec::new();
        for profile_json in profiles_data {
            let profile: PlayerProfile = serde_json::from_str(&profile_json)
                .map_err(|err| RepositoryError::InternalError(err.to_string()))?;
            profiles.push(profile);
        }
        Ok(profiles)
    }

    async fn save(&mut self, profile: PlayerProfile) -> Result<PlayerProfile, RepositoryError> {
        let mut connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| RepositoryError::InternalError(err.to_string()))?;
        let profile_json = serde_json::to_string(&profile)
            .map_err(|err| RepositoryError::InternalError(err.to_string()))?;
        redis::cmd("HSET")
            .arg(PROFILES_HASH)
            .arg(&profile.id)
            .arg(profile_json)
            .query_async(&mut connection)
            .await
            .map_err(|err| RepositoryError::InternalError(err.to_string()))?;
        Ok(profile)
    }
}
