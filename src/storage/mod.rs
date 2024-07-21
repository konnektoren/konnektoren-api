mod error;
mod leaderboard_repository;
mod memory_repository;
mod profile_repository;

pub trait Storage: ProfileRepository + LeaderboardRepository {}

#[cfg(feature = "redis")]
mod redis_storage;

pub use error::RepositoryError;
pub use leaderboard_repository::LeaderboardRepository;
pub use memory_repository::MemoryRepository;
pub use profile_repository::ProfileRepository;

#[cfg(feature = "redis")]
pub use redis_storage::RedisStorage;
