use crate::compatibility::LegacyPerformanceRecord;
use crate::storage::{
    CouponRepository, LeaderboardRepository, ProfileRepository, RepositoryError, ReviewRepository,
    Storage, WindowedCounterRepository,
};
use async_trait::async_trait;
use konnektoren_core::challenges::{PerformanceRecord, Review};
use konnektoren_core::prelude::{Coupon, PlayerProfile};
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

const USER_COUNTER_KEY: &str = "user_counter";
const WINDOW_SECONDS: i64 = 24 * 60 * 60;

const COUPONS_HSET: &str = "coupons";

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

        // Get all keys and values
        let records: Vec<(String, String)> = redis::cmd("HGETALL")
            .arg(&hset)
            .query_async(&mut connection)
            .await
            .map_err(|err| RepositoryError::InternalError(err.to_string()))?;

        let mut performance_records = Vec::new();
        let mut updates_needed = Vec::new();

        for (key, record_json) in records {
            // Try to deserialize as PerformanceRecord first
            if let Ok(record) = serde_json::from_str::<PerformanceRecord>(&record_json) {
                performance_records.push(record);
                continue;
            }

            // If that fails, try as LegacyPerformanceRecord
            if let Ok(legacy_record) = serde_json::from_str::<LegacyPerformanceRecord>(&record_json)
            {
                let converted_record: PerformanceRecord = legacy_record.into();

                // Store the key and the new format for updating
                updates_needed.push((key, converted_record.clone()));
                performance_records.push(converted_record);
                continue;
            }

            return Err(RepositoryError::InternalError(
                "Invalid Performance Record format".to_string(),
            ));
        }

        // Update any legacy records in the storage
        if !updates_needed.is_empty() {
            for (key, record) in updates_needed {
                let record_json = serde_json::to_string(&record)
                    .map_err(|err| RepositoryError::InternalError(err.to_string()))?;

                redis::cmd("HSET")
                    .arg(&hset)
                    .arg(key)
                    .arg(record_json)
                    .query_async(&mut connection)
                    .await
                    .map_err(|err| RepositoryError::InternalError(err.to_string()))?;
            }
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
        let mut connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| RepositoryError::InternalError(err.to_string()))?;

        // Get all records with their keys
        let records: Vec<(String, String)> = redis::cmd("HGETALL")
            .arg(&hset)
            .query_async(&mut connection)
            .await
            .map_err(|err| RepositoryError::InternalError(err.to_string()))?;

        // Find the matching record and its key
        for (key, record_json) in records {
            let record = if let Ok(record) = serde_json::from_str::<PerformanceRecord>(&record_json)
            {
                record
            } else if let Ok(legacy_record) =
                serde_json::from_str::<LegacyPerformanceRecord>(&record_json)
            {
                legacy_record.into()
            } else {
                continue;
            };

            // Compare relevant fields to find a match
            if record.profile_name == performance_record.profile_name
                && record.performance_percentage == performance_record.performance_percentage
                && record.date == performance_record.date
                && record.challenges_performance == performance_record.challenges_performance
            {
                // Remove the record using its key
                redis::cmd("HDEL")
                    .arg(&hset)
                    .arg(key)
                    .query_async(&mut connection)
                    .await
                    .map_err(|err| RepositoryError::InternalError(err.to_string()))?;

                return Ok(performance_record);
            }
        }

        Err(RepositoryError::NotFound(
            "No matching record found".to_string(),
        ))
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

        let review_keys: Vec<String> = conn
            .keys(format!("{}:*", REVIEWS_HSET))
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        let mut all_reviews = Vec::new();

        for key in review_keys {
            let namespace = key.trim_start_matches(&format!("{}:", REVIEWS_HSET));
            let reviews = self.fetch_reviews(namespace).await?;
            all_reviews.extend(reviews);
        }

        Ok(all_reviews)
    }
}

#[async_trait]
impl CouponRepository for RedisStorage {
    async fn fetch(&self, coupon_code: &str) -> Result<Option<Coupon>, RepositoryError> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        let coupon_json: Option<String> = conn
            .hget(COUPONS_HSET, coupon_code)
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        match coupon_json {
            Some(json) => {
                let coupon = serde_json::from_str(&json)
                    .map_err(|e| RepositoryError::InternalError(e.to_string()))?;
                Ok(Some(coupon))
            }
            None => Ok(None),
        }
    }

    async fn fetch_all(&self) -> Result<Vec<Coupon>, RepositoryError> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        let coupons_data: Vec<String> = conn
            .hvals(COUPONS_HSET)
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        let coupons = coupons_data
            .into_iter()
            .map(|data| {
                serde_json::from_str(&data)
                    .map_err(|e| RepositoryError::InternalError(e.to_string()))
            })
            .collect::<Result<Vec<Coupon>, RepositoryError>>()?;

        Ok(coupons)
    }

    async fn save(&mut self, coupon: Coupon) -> Result<Coupon, RepositoryError> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        let coupon_json = serde_json::to_string(&coupon)
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        conn.hset(COUPONS_HSET, &coupon.code, &coupon_json)
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        Ok(coupon)
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

#[async_trait]
impl WindowedCounterRepository for RedisStorage {
    async fn get_active_count(&self, namespace: &str) -> Result<u32, RepositoryError> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        let key = format!("{}:{}", USER_COUNTER_KEY, namespace);
        let count: u32 = conn
            .zcount(&key, "-inf", "+inf")
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        Ok(count)
    }

    async fn record_presence(&mut self, namespace: &str) -> Result<u32, RepositoryError> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        let key = format!("{}:{}", USER_COUNTER_KEY, namespace);
        let timestamp = chrono::Utc::now().timestamp() as f64;

        // Add presence with current timestamp
        let _: () = conn
            .zadd(&key, timestamp.to_string(), timestamp)
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        // Remove old entries (older than 24 hours)
        let min_timestamp = timestamp - WINDOW_SECONDS as f64;
        let _: () = conn
            .zrevrangebyscore(&key, "-inf", min_timestamp)
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        // Set expiration on the entire key
        let _: () = conn
            .expire(&key, WINDOW_SECONDS)
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        // Get updated count
        self.get_active_count(namespace).await
    }
}
