use crate::storage::error::RepositoryError;
use async_trait::async_trait;
use konnektoren_core::prelude::PlayerProfile;

#[async_trait]
pub trait ProfileRepository: Send + Sync {
    async fn fetch(&self, profile_id: String) -> Result<PlayerProfile, RepositoryError>;
    async fn save(&mut self, profile: PlayerProfile) -> Result<PlayerProfile, RepositoryError>;
}
