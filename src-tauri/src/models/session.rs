use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WritingSession {
    pub id: String,
    pub start: String,
    #[serde(default)]
    pub end: Option<String>,
    #[serde(default)]
    pub duration_minutes: Option<f64>,
    #[serde(default)]
    pub words_written: u32,
    pub chapter_slug: String,
    #[serde(default)]
    pub sprint_goal: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStats {
    pub total_sessions: u32,
    pub total_words: u64,
    pub total_minutes: f64,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub daily_average: f64,
    pub weekly_average: f64,
    pub monthly_average: f64,
    pub best_day_words: u32,
    pub best_day_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsData {
    #[serde(default)]
    pub sessions: Vec<WritingSession>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writing_session_yaml_round_trip() {
        let session = WritingSession {
            id: "2026-02-14T10:30:00Z".to_string(),
            start: "2026-02-14T10:30:00Z".to_string(),
            end: Some("2026-02-14T11:00:00Z".to_string()),
            duration_minutes: Some(30.0),
            words_written: 847,
            chapter_slug: "chapter-1".to_string(),
            sprint_goal: Some(500),
        };

        let yaml = serde_yaml::to_string(&session).unwrap();
        let deserialized: WritingSession = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(deserialized.id, session.id);
        assert_eq!(deserialized.start, session.start);
        assert_eq!(deserialized.end, session.end);
        assert_eq!(deserialized.duration_minutes, session.duration_minutes);
        assert_eq!(deserialized.words_written, session.words_written);
        assert_eq!(deserialized.chapter_slug, session.chapter_slug);
        assert_eq!(deserialized.sprint_goal, session.sprint_goal);
    }

    #[test]
    fn writing_session_deserializes_with_optional_fields_missing() {
        let yaml = r#"
id: "2026-02-14T10:30:00Z"
start: "2026-02-14T10:30:00Z"
chapterSlug: chapter-1
"#;
        let session: WritingSession = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(session.id, "2026-02-14T10:30:00Z");
        assert!(session.end.is_none());
        assert!(session.duration_minutes.is_none());
        assert_eq!(session.words_written, 0);
        assert!(session.sprint_goal.is_none());
    }

    #[test]
    fn sessions_data_yaml_round_trip() {
        let data = SessionsData {
            sessions: vec![
                WritingSession {
                    id: "2026-02-14T10:30:00Z".to_string(),
                    start: "2026-02-14T10:30:00Z".to_string(),
                    end: Some("2026-02-14T11:00:00Z".to_string()),
                    duration_minutes: Some(30.0),
                    words_written: 847,
                    chapter_slug: "chapter-1".to_string(),
                    sprint_goal: None,
                },
                WritingSession {
                    id: "2026-02-15T09:00:00Z".to_string(),
                    start: "2026-02-15T09:00:00Z".to_string(),
                    end: None,
                    duration_minutes: None,
                    words_written: 0,
                    chapter_slug: "chapter-2".to_string(),
                    sprint_goal: Some(1000),
                },
            ],
        };

        let yaml = serde_yaml::to_string(&data).unwrap();
        let deserialized: SessionsData = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(deserialized.sessions.len(), 2);
        assert_eq!(deserialized.sessions[0].words_written, 847);
        assert_eq!(deserialized.sessions[1].sprint_goal, Some(1000));
    }

    #[test]
    fn sessions_data_deserializes_empty_list() {
        let yaml = "sessions: []\n";
        let data: SessionsData = serde_yaml::from_str(yaml).unwrap();
        assert!(data.sessions.is_empty());
    }

    #[test]
    fn session_stats_serializes_all_fields() {
        let stats = SessionStats {
            total_sessions: 10,
            total_words: 5000,
            total_minutes: 300.0,
            current_streak: 3,
            longest_streak: 7,
            daily_average: 500.0,
            weekly_average: 3500.0,
            monthly_average: 15000.0,
            best_day_words: 1200,
            best_day_date: Some("2026-02-10".to_string()),
        };

        let yaml = serde_yaml::to_string(&stats).unwrap();
        assert!(yaml.contains("totalSessions: 10"));
        assert!(yaml.contains("totalWords: 5000"));
        assert!(yaml.contains("currentStreak: 3"));
        assert!(yaml.contains("longestStreak: 7"));
        assert!(yaml.contains("bestDayWords: 1200"));
        assert!(yaml.contains("bestDayDate"));
    }

    #[test]
    fn session_stats_with_no_best_day() {
        let stats = SessionStats {
            total_sessions: 0,
            total_words: 0,
            total_minutes: 0.0,
            current_streak: 0,
            longest_streak: 0,
            daily_average: 0.0,
            weekly_average: 0.0,
            monthly_average: 0.0,
            best_day_words: 0,
            best_day_date: None,
        };

        let yaml = serde_yaml::to_string(&stats).unwrap();
        let deserialized: SessionStats = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(deserialized.total_sessions, 0);
        assert!(deserialized.best_day_date.is_none());
    }
}
