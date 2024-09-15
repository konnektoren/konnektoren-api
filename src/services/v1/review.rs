use crate::storage::Storage;
use anyhow::Error;
use konnektoren_core::challenges::Review;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn fetch_reviews(
    challenge_id: String,
    repository: Arc<Mutex<dyn Storage>>,
) -> Result<Vec<Review>, Error> {
    let reviews = repository
        .lock()
        .await
        .fetch_reviews(&challenge_id)
        .await
        .map_err(|err| {
            log::error!(
                "Error fetching reviews for challenge {}: {:?}",
                challenge_id,
                err
            );
            err
        })?;
    log::debug!(
        "Returning reviews for challenge {}: {:?}",
        challenge_id,
        reviews
    );
    Ok(reviews)
}

pub async fn fetch_average_rating(
    challenge_id: String,
    repository: Arc<Mutex<dyn Storage>>,
) -> Result<f64, Error> {
    let average_rating = repository
        .lock()
        .await
        .fetch_average_rating(&challenge_id)
        .await
        .map_err(|err| {
            log::error!(
                "Error fetching average rating for challenge {}: {:?}",
                challenge_id,
                err
            );
            err
        })?;

    log::debug!(
        "Returning average rating for challenge {}: {:?}",
        challenge_id,
        average_rating
    );
    Ok(average_rating)
}

pub async fn store_review(
    review: Review,
    repository: Arc<Mutex<dyn Storage>>,
) -> Result<(), Error> {
    log::debug!("Received review to store: {:?}", review);
    let result = repository
        .lock()
        .await
        .store_review(review)
        .await
        .map_err(|err| {
            log::error!("Error storing review: {:?}", err);
            err
        })?;
    log::debug!("Stored review successfully");
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryRepository;
    use konnektoren_core::challenges::Review;

    #[tokio::test]
    async fn test_store_review() {
        let repository = Arc::new(Mutex::new(MemoryRepository::new()));
        let review = Review {
            challenge_id: "example_challenge_id".to_string(),
            rating: 5,
            comment: Some("Great challenge!".to_string()),
        };

        store_review(review.clone(), repository.clone())
            .await
            .expect("Failed to store review");

        let reviews = fetch_reviews("example_challenge_id".to_string(), repository.clone())
            .await
            .expect("Failed to fetch reviews");

        assert_eq!(reviews.len(), 1);
        assert_eq!(reviews[0], review);
    }

    #[tokio::test]
    async fn test_fetch_average_rating() {
        let repository = Arc::new(Mutex::new(MemoryRepository::new()));
        let review1 = Review {
            challenge_id: "example_challenge_id".to_string(),
            rating: 5,
            comment: Some("Great challenge!".to_string()),
        };
        let review2 = Review {
            challenge_id: "example_challenge_id".to_string(),
            rating: 3,
            comment: Some("Good challenge!".to_string()),
        };

        store_review(review1.clone(), repository.clone())
            .await
            .expect("Failed to store review");
        store_review(review2.clone(), repository.clone())
            .await
            .expect("Failed to store review");

        let average_rating =
            fetch_average_rating("example_challenge_id".to_string(), repository.clone())
                .await
                .expect("Failed to fetch average rating");

        assert_eq!(average_rating, 4.0);
    }
}
