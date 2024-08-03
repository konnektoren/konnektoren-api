use crate::storage::leaderboard_repository::LeaderboardRepository;
use crate::storage::{ProfileRepository, RepositoryError, Storage};
use async_trait::async_trait;
use konnektoren_core::challenges::PerformanceRecord;
use konnektoren_core::prelude::PlayerProfile;
use std::hash::{DefaultHasher, Hash, Hasher};

pub struct RedisStorage {
    client: redis::Client,
}

const PROFILES_HSET: &str = "profiles";
const PERFORMANCE_RECORDS_HSET: &str = "performance_records";
const PERFORMANCE_RECORDS_LIMIT: usize = 10;

impl RedisStorage {
    pub fn new(url: &str) -> Self {
        Self {
            client: redis::Client::open(url).expect("Invalid Redis URL"),
        }
    }
}

impl Storage for RedisStorage {}

#[async_trait]
impl ProfileRepository for RedisStorage {
    async fn fetch(&self, profile_id: String) -> Result<PlayerProfile, RepositoryError> {
        let mut connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| RepositoryError::InternalError(err.to_string()))?;
        let profile_json: String = redis::cmd("HGET")
            .arg(PROFILES_HSET)
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
            .arg(PROFILES_HSET)
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
            .arg(PROFILES_HSET)
            .arg(&profile.id)
            .arg(profile_json)
            .query_async(&mut connection)
            .await
            .map_err(|err| RepositoryError::InternalError(err.to_string()))?;
        Ok(profile)
    }
}

#[async_trait]
impl LeaderboardRepository for RedisStorage {
    async fn fetch_performance_records(
        &self,
        namespace: &str,
    ) -> Result<Vec<PerformanceRecord>, RepositoryError> {
        let hset = format!("{}:{}", PERFORMANCE_RECORDS_HSET, namespace);
        let mut connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| RepositoryError::InternalError(err.to_string()))?;
        let performance_records_data: Vec<String> = redis::cmd("HVALS")
            .arg(hset)
            .query_async(&mut connection)
            .await
            .map_err(|err| RepositoryError::InternalError(err.to_string()))?;
        let mut performance_records = Vec::new();
        for performance_record_json in performance_records_data {
            let performance_record: PerformanceRecord =
                serde_json::from_str(&performance_record_json)
                    .map_err(|err| RepositoryError::InternalError(err.to_string()))?;
            performance_records.push(performance_record);
        }
        Ok(performance_records)
    }

    async fn add_performance_record(
        &mut self,
        namespace: &str,
        performance_record: PerformanceRecord,
    ) -> Result<PerformanceRecord, RepositoryError> {
        let hset = format!("{}:{}", PERFORMANCE_RECORDS_HSET, namespace);
        let mut connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| RepositoryError::InternalError(err.to_string()))?;
        let performance_record_json = serde_json::to_string(&performance_record)
            .map_err(|err| RepositoryError::InternalError(err.to_string()))?;
        let performance_records_count: usize = redis::cmd("HLEN")
            .arg(&hset)
            .query_async(&mut connection)
            .await
            .map_err(|err| RepositoryError::InternalError(err.to_string()))?;
        if performance_records_count < PERFORMANCE_RECORDS_LIMIT {
            let mut hasher = DefaultHasher::default();
            performance_record.hash(&mut hasher);
            let id = hasher.finish().to_string();

            redis::cmd("hset")
                .arg(&hset)
                .arg(id)
                .arg(performance_record_json)
                .query_async(&mut connection)
                .await
                .map_err(|err| RepositoryError::InternalError(err.to_string()))?;
            Ok(performance_record)
        } else {
            Err(RepositoryError::LimitReached(PERFORMANCE_RECORDS_LIMIT))
        }
    }

    async fn remove_performance_record(
        &mut self,
        namespace: &str,
        performance_record: PerformanceRecord,
    ) -> Result<PerformanceRecord, RepositoryError> {
        let hset = format!("{}:{}", PERFORMANCE_RECORDS_HSET, namespace);
        let mut hasher = DefaultHasher::default();
        performance_record.hash(&mut hasher);
        let id = hasher.finish().to_string();

        let mut connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| RepositoryError::InternalError(err.to_string()))?;

        let performance_record_json: String = redis::cmd("HGET")
            .arg(&hset)
            .arg(id.clone())
            .query_async(&mut connection)
            .await
            .map_err(|err| RepositoryError::InternalError(err.to_string()))?;
        if performance_record_json.is_empty() {
            return Err(RepositoryError::NotFound);
        } else {
            redis::cmd("HDEL")
                .arg(&hset)
                .arg(id.clone())
                .query_async(&mut connection)
                .await
                .map_err(|err| RepositoryError::InternalError(err.to_string()))?;
            Ok(performance_record)
        }
    }
}
