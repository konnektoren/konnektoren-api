mod error;
mod leaderboard_repository;
mod memory_repository;
mod profile_repository;
mod review_repository;
mod windowed_counter_repository;

#[cfg(not(feature = "chat"))]
pub trait Storage: ProfileRepository + LeaderboardRepository + ReviewRepository {}

#[cfg(feature = "chat")]
pub trait Storage:
    ProfileRepository
    + LeaderboardRepository
    + ReviewRepository
    + MessageStorage
    + WindowedCounterRepository
{
}

#[cfg(feature = "redis")]
mod redis_storage;

pub use error::RepositoryError;
pub use leaderboard_repository::LeaderboardRepository;
pub use memory_repository::MemoryRepository;
pub use profile_repository::ProfileRepository;
pub use review_repository::ReviewRepository;
pub use windowed_counter_repository::WindowedCounterRepository;
use yew_chat::server::MessageStorage;

#[cfg(feature = "redis")]
pub use redis_storage::RedisStorage;
