use anyhow::Error;
use konnektoren_core::prelude::PlayerProfile;

pub async fn fetch_profile(profile_id: String) -> Result<PlayerProfile, Error> {
    let profile = PlayerProfile::new(profile_id);
    log::info!("Returning profile: {:?}", profile);
    Ok(profile)
}

pub async fn save_profile(profile: PlayerProfile) -> Result<PlayerProfile, Error> {
    log::info!("Received profile: {:?}", profile);
    Ok(profile)
}
