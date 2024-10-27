use chrono::{DateTime, Utc};
use konnektoren_core::challenges::performance_record::{
    ChallengeId, ChallengePercentage, PerformanceRecord,
};
use serde::{Deserialize, Serialize};

// New type for compatibility
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash)]
pub struct LegacyPerformanceRecord {
    pub game_path_id: String,
    pub profile_name: String,
    pub challenges_performance: Vec<(ChallengeId, ChallengePercentage)>,
    pub total_challenges: usize,
    pub performance_percentage: u8,
    pub date: DateTime<Utc>,
}

impl Into<PerformanceRecord> for LegacyPerformanceRecord {
    fn into(self) -> PerformanceRecord {
        PerformanceRecord {
            game_path_id: self.game_path_id.clone(),
            profile_name: self.profile_name.clone(),
            challenges_performance: self
                .challenges_performance
                .iter()
                .map(|(id, performance)| (id.clone(), *performance, 3600 * 1000)) // Add time as 1h
                .collect(),
            total_challenges: self.total_challenges,
            performance_percentage: self.performance_percentage,
            date: self.date,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_performance_record_to_performance_record() {
        let legacy_record = LegacyPerformanceRecord {
            game_path_id: "game_path_id".to_string(),
            profile_name: "profile_name".to_string(),
            challenges_performance: vec![("challenge_id".to_string(), 100)],
            total_challenges: 1,
            performance_percentage: 100,
            date: Utc::now(),
        };

        let performance_record: PerformanceRecord = legacy_record.into();

        assert_eq!(performance_record.game_path_id, "game_path_id");
        assert_eq!(performance_record.profile_name, "profile_name");
        assert_eq!(
            performance_record.challenges_performance,
            vec![("challenge_id".to_string(), 100, 3600000)]
        );
        assert_eq!(performance_record.total_challenges, 1);
        assert_eq!(performance_record.performance_percentage, 100);
    }
}
