use crate::storage::{
    LeaderboardRepository, ProfileRepository, RepositoryError, ReviewRepository, Storage,
};
use async_trait::async_trait;
use konnektoren_core::challenges::{PerformanceRecord, Review};
use konnektoren_core::prelude::PlayerProfile;
use redis::AsyncCommands;
use std::hash::{DefaultHasher, Hash, Hasher};
use yew_chat::prelude::{Message, MessageReceiver, MessageSender, ReceiveError, SendError};
use yew_chat::server::MessageStorage;

pub struct RedisStorage {
    client: redis::Client,
}

const PROFILES_HSET: &str = "profiles";
const PERFORMANCE_RECORDS_HSET: &str = "performance_records";
const PERFORMANCE_RECORDS_LIMIT: usize = 10;

const REVIEWS_HSET: &str = "reviews";

const CHAT_MESSAGES_HSET: &str = "chat_messages";

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
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        let profile_json: String = conn
            .hget(PROFILES_HSET, &profile_id)
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;
        serde_json::from_str(&profile_json)
            .map_err(|e| RepositoryError::InternalError(e.to_string()))
    }

    async fn fetch_all(&self) -> Result<Vec<PlayerProfile>, RepositoryError> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;
        let profiles_data: Vec<String> = conn
            .hvals(PROFILES_HSET)
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;
        profiles_data
            .into_iter()
            .map(|data| {
                serde_json::from_str(&data)
                    .map_err(|e| RepositoryError::InternalError(e.to_string()))
            })
            .collect()
    }

    async fn save(&mut self, profile: PlayerProfile) -> Result<PlayerProfile, RepositoryError> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;
        let profile_json = serde_json::to_string(&profile)
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;
        conn.hset(PROFILES_HSET, &profile.id, &profile_json)
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;
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

#[async_trait]
impl ReviewRepository for RedisStorage {
    async fn store_review(&mut self, review: Review) -> Result<(), RepositoryError> {
        let ns_key = format!("{}:{}", REVIEWS_HSET, review.challenge_id);
        let review_json = serde_json::to_string(&review)
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        let _: () = conn
            .rpush::<&str, _, ()>(&ns_key, &review_json)
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn fetch_reviews(&self, namespace: &str) -> Result<Vec<Review>, RepositoryError> {
        let ns_key = format!("{}:{}", REVIEWS_HSET, namespace);
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        let review_jsons: Vec<String> = conn
            .lrange(&ns_key, 0, -1)
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        review_jsons
            .iter()
            .map(|json| {
                serde_json::from_str::<Review>(json)
                    .map_err(|e| RepositoryError::InternalError(e.to_string()))
            })
            .collect()
    }

    async fn fetch_average_rating(&self, namespace: &str) -> Result<f64, RepositoryError> {
        let reviews = self.fetch_reviews(namespace).await?;

        if reviews.is_empty() {
            Ok(0.0)
        } else {
            let total: u32 = reviews.iter().map(|r| r.rating as u32).sum();
            let avg_rating = total as f64 / reviews.len() as f64;
            Ok(avg_rating)
        }
    }

    async fn fetch_all_reviews(&self) -> Result<Vec<Review>, RepositoryError> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        let review_jsons: Vec<String> = conn
            .hvals(REVIEWS_HSET)
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        review_jsons
            .iter()
            .map(|json| {
                serde_json::from_str::<Review>(json)
                    .map_err(|e| RepositoryError::InternalError(e.to_string()))
            })
            .collect()
    }
}

#[cfg(feature = "chat")]
#[async_trait]
impl MessageReceiver for RedisStorage {
    async fn receive_messages(&self, channel: &str) -> Result<Vec<Message>, ReceiveError> {
        let hset = format!("{}:{}", CHAT_MESSAGES_HSET, channel);
        let mut connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| ReceiveError::InternalError(err.to_string()))?;
        let messages_data: Vec<String> = redis::cmd("HVALS")
            .arg(hset)
            .query_async(&mut connection)
            .await
            .map_err(|err| ReceiveError::InternalError(err.to_string()))?;
        let mut messages = Vec::new();
        for message_json in messages_data {
            let message: Message = serde_json::from_str(&message_json)
                .map_err(|err| ReceiveError::InternalError(err.to_string()))?;
            messages.push(message);
        }
        Ok(messages)
    }
}

#[cfg(feature = "chat")]
#[async_trait]
impl MessageSender for RedisStorage {
    async fn send_message(&self, channel: &str, message: Message) -> Result<(), SendError> {
        let hset = format!("{}:{}", CHAT_MESSAGES_HSET, channel);
        let key = format!("{}:{}", message.sender, message.timestamp);
        let message_json = serde_json::to_string(&message)
            .map_err(|err| SendError::InternalError(err.to_string()))?;
        let mut connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| SendError::InternalError(err.to_string()))?;
        redis::cmd("hset")
            .arg(hset)
            .arg(key)
            .arg(message_json)
            .query_async(&mut connection)
            .await
            .map_err(|err| SendError::InternalError(err.to_string()))?;
        Ok(())
    }
}

#[cfg(feature = "chat")]
impl MessageStorage for RedisStorage {}
