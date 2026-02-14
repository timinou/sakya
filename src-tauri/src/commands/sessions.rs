use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use chrono::{NaiveDate, Utc};

use crate::error::AppError;
use crate::models::session::{SessionStats, SessionsData, WritingSession};
use crate::services::yaml_service::{read_yaml, write_yaml};

/// Path to the sessions data file within a project.
fn sessions_path(project_path: &str) -> PathBuf {
    PathBuf::from(project_path)
        .join(".sakya")
        .join("sessions.yaml")
}

/// Load sessions data from the project's sessions.yaml.
/// Returns an empty SessionsData if the file does not exist yet.
fn load_sessions(project_path: &str) -> Result<SessionsData, AppError> {
    let path = sessions_path(project_path);
    if !path.exists() {
        return Ok(SessionsData {
            sessions: Vec::new(),
        });
    }
    read_yaml(&path)
}

/// Save sessions data to the project's sessions.yaml.
fn save_sessions(project_path: &str, data: &SessionsData) -> Result<(), AppError> {
    let path = sessions_path(project_path);
    write_yaml(&path, data)
}

/// Calculate aggregated statistics from a slice of sessions.
/// This is a pure function with no side effects.
fn calculate_stats(sessions: &[WritingSession]) -> SessionStats {
    if sessions.is_empty() {
        return SessionStats {
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
    }

    let total_sessions = sessions.len() as u32;
    let total_words: u64 = sessions.iter().map(|s| s.words_written as u64).sum();
    let total_minutes: f64 = sessions.iter().filter_map(|s| s.duration_minutes).sum();

    // Aggregate words per day (using the start date)
    let mut daily_words: BTreeMap<NaiveDate, u32> = BTreeMap::new();
    let mut session_dates: BTreeSet<NaiveDate> = BTreeSet::new();

    for session in sessions {
        if let Ok(dt) = session.start.parse::<chrono::DateTime<Utc>>() {
            let date = dt.date_naive();
            *daily_words.entry(date).or_insert(0) += session.words_written;
            session_dates.insert(date);
        }
    }

    // Best day
    let (best_day_date, best_day_words) = daily_words
        .iter()
        .max_by_key(|(_, &words)| words)
        .map(|(date, &words)| (Some(date.format("%Y-%m-%d").to_string()), words))
        .unwrap_or((None, 0));

    // Streak calculation
    let today = Utc::now().date_naive();
    let sorted_dates: Vec<NaiveDate> = session_dates.into_iter().collect();

    let current_streak = calculate_current_streak(&sorted_dates, today);
    let longest_streak = calculate_longest_streak(&sorted_dates);

    // Averages: based on the span from the first session date to today
    let (daily_average, weekly_average, monthly_average) = if !sorted_dates.is_empty() {
        let first_date = sorted_dates[0];
        let days_span = (today - first_date).num_days().max(1) as f64;
        let daily_avg = total_words as f64 / days_span;
        let weekly_avg = daily_avg * 7.0;
        let monthly_avg = daily_avg * 30.0;
        (daily_avg, weekly_avg, monthly_avg)
    } else {
        (0.0, 0.0, 0.0)
    };

    SessionStats {
        total_sessions,
        total_words,
        total_minutes,
        current_streak,
        longest_streak,
        daily_average,
        weekly_average,
        monthly_average,
        best_day_words,
        best_day_date,
    }
}

/// Calculate the current streak: consecutive calendar days counting backwards
/// from today where at least one session was recorded.
fn calculate_current_streak(sorted_dates: &[NaiveDate], today: NaiveDate) -> u32 {
    if sorted_dates.is_empty() {
        return 0;
    }

    let date_set: BTreeSet<NaiveDate> = sorted_dates.iter().copied().collect();

    // Start from today and count backwards
    let mut streak = 0u32;
    let mut check_date = today;

    // If today has no session, check yesterday (allow a 1-day grace for "current" streak)
    if !date_set.contains(&check_date) {
        check_date -= chrono::Duration::days(1);
        if !date_set.contains(&check_date) {
            return 0;
        }
    }

    while date_set.contains(&check_date) {
        streak += 1;
        check_date -= chrono::Duration::days(1);
    }

    streak
}

/// Calculate the longest streak ever: the maximum number of consecutive
/// calendar days with at least one session.
fn calculate_longest_streak(sorted_dates: &[NaiveDate]) -> u32 {
    if sorted_dates.is_empty() {
        return 0;
    }

    let unique_dates: BTreeSet<NaiveDate> = sorted_dates.iter().copied().collect();
    let unique_sorted: Vec<NaiveDate> = unique_dates.into_iter().collect();

    let mut longest = 1u32;
    let mut current = 1u32;

    for i in 1..unique_sorted.len() {
        if unique_sorted[i] - unique_sorted[i - 1] == chrono::Duration::days(1) {
            current += 1;
            if current > longest {
                longest = current;
            }
        } else {
            current = 1;
        }
    }

    longest
}

/// Start a new writing session. Creates the sessions file if it doesn't exist.
/// Returns the session ID (ISO 8601 timestamp).
#[tauri::command]
pub fn start_session(
    project_path: &str,
    chapter_slug: &str,
    sprint_goal: Option<u32>,
) -> Result<String, AppError> {
    let now = Utc::now();
    let id = now.to_rfc3339();

    let session = WritingSession {
        id: id.clone(),
        start: id.clone(),
        end: None,
        duration_minutes: None,
        words_written: 0,
        chapter_slug: chapter_slug.to_string(),
        sprint_goal,
    };

    let mut data = load_sessions(project_path)?;
    data.sessions.push(session);
    save_sessions(project_path, &data)?;

    Ok(id)
}

/// End an existing writing session by ID. Sets end time, calculates duration,
/// and records word count.
#[tauri::command]
pub fn end_session(
    project_path: &str,
    session_id: &str,
    words_written: u32,
) -> Result<(), AppError> {
    let mut data = load_sessions(project_path)?;

    let session = data
        .sessions
        .iter_mut()
        .find(|s| s.id == session_id)
        .ok_or_else(|| AppError::NotFound(format!("Session not found: {}", session_id)))?;

    let now = Utc::now();
    let end_time = now.to_rfc3339();

    // Calculate duration from start to now
    if let Ok(start_dt) = session.start.parse::<chrono::DateTime<Utc>>() {
        let duration = now - start_dt;
        session.duration_minutes = Some(duration.num_seconds() as f64 / 60.0);
    }

    session.end = Some(end_time);
    session.words_written = words_written;

    save_sessions(project_path, &data)?;
    Ok(())
}

/// Get writing sessions, optionally filtered by date range.
/// `from` and `to` are ISO 8601 date strings (e.g. "2026-02-01").
#[tauri::command]
pub fn get_sessions(
    project_path: &str,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<Vec<WritingSession>, AppError> {
    let data = load_sessions(project_path)?;

    let from_date = from.and_then(|f| NaiveDate::parse_from_str(f, "%Y-%m-%d").ok());
    let to_date = to.and_then(|t| NaiveDate::parse_from_str(t, "%Y-%m-%d").ok());

    let filtered: Vec<WritingSession> = data
        .sessions
        .into_iter()
        .filter(|session| {
            let session_date = session
                .start
                .parse::<chrono::DateTime<Utc>>()
                .ok()
                .map(|dt| dt.date_naive());

            match session_date {
                Some(date) => {
                    let after_from = from_date.is_none_or(|f| date >= f);
                    let before_to = to_date.is_none_or(|t| date <= t);
                    after_from && before_to
                }
                None => true, // Include sessions with unparseable dates
            }
        })
        .collect();

    Ok(filtered)
}

/// Get aggregated session statistics for the project.
#[tauri::command]
pub fn get_session_stats(project_path: &str) -> Result<SessionStats, AppError> {
    let data = load_sessions(project_path)?;
    Ok(calculate_stats(&data.sessions))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::setup_test_project;

    // ── Helper ──────────────────────────────────────────────────────

    /// Create a test project with a .sakya directory already present.
    fn setup_session_test() -> (tempfile::TempDir, String) {
        let (dir, root) = setup_test_project();
        // Ensure .sakya directory exists
        std::fs::create_dir_all(root.join(".sakya")).unwrap();
        let path = root.to_str().unwrap().to_string();
        (dir, path)
    }

    /// Write a sessions.yaml directly for testing.
    fn write_test_sessions(project_path: &str, sessions: Vec<WritingSession>) {
        let data = SessionsData { sessions };
        save_sessions(project_path, &data).unwrap();
    }

    // ── start_session ───────────────────────────────────────────────

    #[test]
    fn start_session_creates_file_if_missing() {
        let (_dir, path) = setup_session_test();

        let sessions_file = sessions_path(&path);
        assert!(!sessions_file.exists());

        let id = start_session(&path, "chapter-1", None).unwrap();
        assert!(!id.is_empty());
        assert!(sessions_file.exists());
    }

    #[test]
    fn start_session_returns_valid_iso8601_id() {
        let (_dir, path) = setup_session_test();

        let id = start_session(&path, "chapter-1", None).unwrap();

        // Should parse as a valid DateTime
        let parsed = id.parse::<chrono::DateTime<Utc>>();
        assert!(parsed.is_ok(), "ID should be valid ISO 8601: {}", id);
    }

    #[test]
    fn start_session_stores_session_in_file() {
        let (_dir, path) = setup_session_test();

        let id = start_session(&path, "chapter-1", Some(500)).unwrap();

        let data = load_sessions(&path).unwrap();
        assert_eq!(data.sessions.len(), 1);

        let session = &data.sessions[0];
        assert_eq!(session.id, id);
        assert_eq!(session.chapter_slug, "chapter-1");
        assert_eq!(session.sprint_goal, Some(500));
        assert!(session.end.is_none());
        assert!(session.duration_minutes.is_none());
        assert_eq!(session.words_written, 0);
    }

    #[test]
    fn start_session_appends_to_existing_sessions() {
        let (_dir, path) = setup_session_test();

        start_session(&path, "chapter-1", None).unwrap();
        start_session(&path, "chapter-2", Some(1000)).unwrap();

        let data = load_sessions(&path).unwrap();
        assert_eq!(data.sessions.len(), 2);
        assert_eq!(data.sessions[0].chapter_slug, "chapter-1");
        assert_eq!(data.sessions[1].chapter_slug, "chapter-2");
    }

    #[test]
    fn start_session_without_sprint_goal() {
        let (_dir, path) = setup_session_test();

        start_session(&path, "chapter-1", None).unwrap();

        let data = load_sessions(&path).unwrap();
        assert!(data.sessions[0].sprint_goal.is_none());
    }

    // ── end_session ─────────────────────────────────────────────────

    #[test]
    fn end_session_sets_end_time_and_duration() {
        let (_dir, path) = setup_session_test();

        let id = start_session(&path, "chapter-1", None).unwrap();
        end_session(&path, &id, 500).unwrap();

        let data = load_sessions(&path).unwrap();
        let session = &data.sessions[0];

        assert!(session.end.is_some());
        assert!(session.duration_minutes.is_some());
        assert_eq!(session.words_written, 500);

        // Duration should be very small since we just started
        let duration = session.duration_minutes.unwrap();
        assert!(
            (0.0..1.0).contains(&duration),
            "Duration should be near zero, got: {}",
            duration
        );
    }

    #[test]
    fn end_session_calculates_duration_correctly() {
        let (_dir, path) = setup_session_test();

        // Write a session that started 30 minutes ago
        let start_time = Utc::now() - chrono::Duration::minutes(30);
        let session = WritingSession {
            id: start_time.to_rfc3339(),
            start: start_time.to_rfc3339(),
            end: None,
            duration_minutes: None,
            words_written: 0,
            chapter_slug: "chapter-1".to_string(),
            sprint_goal: None,
        };
        write_test_sessions(&path, vec![session.clone()]);

        end_session(&path, &session.id, 847).unwrap();

        let data = load_sessions(&path).unwrap();
        let ended = &data.sessions[0];

        assert_eq!(ended.words_written, 847);
        let duration = ended.duration_minutes.unwrap();
        // Should be approximately 30 minutes (allow 1 minute tolerance)
        assert!(
            (duration - 30.0).abs() < 1.0,
            "Duration should be ~30 minutes, got: {}",
            duration
        );
    }

    #[test]
    fn end_session_errors_on_nonexistent_id() {
        let (_dir, path) = setup_session_test();

        start_session(&path, "chapter-1", None).unwrap();

        let result = end_session(&path, "nonexistent-id", 100);
        assert!(result.is_err());

        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("not found") || msg.contains("Not found"),
            "Expected 'not found' error, got: {}",
            msg
        );
    }

    #[test]
    fn end_session_does_not_affect_other_sessions() {
        let (_dir, path) = setup_session_test();

        let id1 = start_session(&path, "chapter-1", None).unwrap();
        let _id2 = start_session(&path, "chapter-2", None).unwrap();

        end_session(&path, &id1, 300).unwrap();

        let data = load_sessions(&path).unwrap();
        assert_eq!(data.sessions[0].words_written, 300);
        assert!(data.sessions[0].end.is_some());
        // Second session should be untouched
        assert_eq!(data.sessions[1].words_written, 0);
        assert!(data.sessions[1].end.is_none());
    }

    // ── get_sessions ────────────────────────────────────────────────

    #[test]
    fn get_sessions_returns_all_when_no_filter() {
        let (_dir, path) = setup_session_test();

        start_session(&path, "chapter-1", None).unwrap();
        start_session(&path, "chapter-2", None).unwrap();

        let sessions = get_sessions(&path, None, None).unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn get_sessions_returns_empty_for_new_project() {
        let (_dir, path) = setup_session_test();

        let sessions = get_sessions(&path, None, None).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn get_sessions_filters_by_date_range() {
        let (_dir, path) = setup_session_test();

        let sessions = vec![
            WritingSession {
                id: "2026-02-10T10:00:00Z".to_string(),
                start: "2026-02-10T10:00:00Z".to_string(),
                end: Some("2026-02-10T10:30:00Z".to_string()),
                duration_minutes: Some(30.0),
                words_written: 300,
                chapter_slug: "chapter-1".to_string(),
                sprint_goal: None,
            },
            WritingSession {
                id: "2026-02-12T10:00:00Z".to_string(),
                start: "2026-02-12T10:00:00Z".to_string(),
                end: Some("2026-02-12T10:30:00Z".to_string()),
                duration_minutes: Some(30.0),
                words_written: 500,
                chapter_slug: "chapter-1".to_string(),
                sprint_goal: None,
            },
            WritingSession {
                id: "2026-02-14T10:00:00Z".to_string(),
                start: "2026-02-14T10:00:00Z".to_string(),
                end: Some("2026-02-14T10:30:00Z".to_string()),
                duration_minutes: Some(30.0),
                words_written: 700,
                chapter_slug: "chapter-2".to_string(),
                sprint_goal: None,
            },
        ];
        write_test_sessions(&path, sessions);

        // Filter: only Feb 11-13 (should get the Feb 12 session)
        let filtered = get_sessions(&path, Some("2026-02-11"), Some("2026-02-13")).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].words_written, 500);
    }

    #[test]
    fn get_sessions_filters_with_only_from() {
        let (_dir, path) = setup_session_test();

        let sessions = vec![
            WritingSession {
                id: "2026-02-10T10:00:00Z".to_string(),
                start: "2026-02-10T10:00:00Z".to_string(),
                end: None,
                duration_minutes: None,
                words_written: 300,
                chapter_slug: "ch-1".to_string(),
                sprint_goal: None,
            },
            WritingSession {
                id: "2026-02-14T10:00:00Z".to_string(),
                start: "2026-02-14T10:00:00Z".to_string(),
                end: None,
                duration_minutes: None,
                words_written: 700,
                chapter_slug: "ch-2".to_string(),
                sprint_goal: None,
            },
        ];
        write_test_sessions(&path, sessions);

        let filtered = get_sessions(&path, Some("2026-02-12"), None).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].words_written, 700);
    }

    #[test]
    fn get_sessions_filters_with_only_to() {
        let (_dir, path) = setup_session_test();

        let sessions = vec![
            WritingSession {
                id: "2026-02-10T10:00:00Z".to_string(),
                start: "2026-02-10T10:00:00Z".to_string(),
                end: None,
                duration_minutes: None,
                words_written: 300,
                chapter_slug: "ch-1".to_string(),
                sprint_goal: None,
            },
            WritingSession {
                id: "2026-02-14T10:00:00Z".to_string(),
                start: "2026-02-14T10:00:00Z".to_string(),
                end: None,
                duration_minutes: None,
                words_written: 700,
                chapter_slug: "ch-2".to_string(),
                sprint_goal: None,
            },
        ];
        write_test_sessions(&path, sessions);

        let filtered = get_sessions(&path, None, Some("2026-02-12")).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].words_written, 300);
    }

    // ── get_session_stats ───────────────────────────────────────────

    #[test]
    fn stats_with_zero_sessions() {
        let stats = calculate_stats(&[]);

        assert_eq!(stats.total_sessions, 0);
        assert_eq!(stats.total_words, 0);
        assert_eq!(stats.total_minutes, 0.0);
        assert_eq!(stats.current_streak, 0);
        assert_eq!(stats.longest_streak, 0);
        assert_eq!(stats.daily_average, 0.0);
        assert_eq!(stats.weekly_average, 0.0);
        assert_eq!(stats.monthly_average, 0.0);
        assert_eq!(stats.best_day_words, 0);
        assert!(stats.best_day_date.is_none());
    }

    #[test]
    fn stats_with_single_session() {
        let today = Utc::now();
        let sessions = vec![WritingSession {
            id: today.to_rfc3339(),
            start: today.to_rfc3339(),
            end: Some((today + chrono::Duration::minutes(25)).to_rfc3339()),
            duration_minutes: Some(25.0),
            words_written: 500,
            chapter_slug: "ch-1".to_string(),
            sprint_goal: None,
        }];

        let stats = calculate_stats(&sessions);

        assert_eq!(stats.total_sessions, 1);
        assert_eq!(stats.total_words, 500);
        assert_eq!(stats.total_minutes, 25.0);
        assert_eq!(stats.best_day_words, 500);
        assert!(stats.best_day_date.is_some());
        // Current streak should be 1 (session today)
        assert_eq!(stats.current_streak, 1);
        assert_eq!(stats.longest_streak, 1);
    }

    #[test]
    fn stats_with_multi_day_streak() {
        let today = Utc::now().date_naive();

        let sessions: Vec<WritingSession> = (0..5)
            .map(|days_ago| {
                let date = today - chrono::Duration::days(days_ago);
                let dt = date.and_hms_opt(10, 0, 0).unwrap().and_utc();
                WritingSession {
                    id: dt.to_rfc3339(),
                    start: dt.to_rfc3339(),
                    end: Some((dt + chrono::Duration::minutes(25)).to_rfc3339()),
                    duration_minutes: Some(25.0),
                    words_written: 400,
                    chapter_slug: "ch-1".to_string(),
                    sprint_goal: None,
                }
            })
            .collect();

        let stats = calculate_stats(&sessions);

        assert_eq!(stats.total_sessions, 5);
        assert_eq!(stats.total_words, 2000);
        assert_eq!(stats.current_streak, 5);
        assert_eq!(stats.longest_streak, 5);
    }

    #[test]
    fn stats_streak_broken_by_gap() {
        let today = Utc::now().date_naive();

        // Sessions today, yesterday, and 4 days ago (gap on day -2 and -3)
        let dates = vec![
            today,
            today - chrono::Duration::days(1),
            today - chrono::Duration::days(4),
        ];

        let sessions: Vec<WritingSession> = dates
            .into_iter()
            .map(|date| {
                let dt = date.and_hms_opt(10, 0, 0).unwrap().and_utc();
                WritingSession {
                    id: dt.to_rfc3339(),
                    start: dt.to_rfc3339(),
                    end: None,
                    duration_minutes: Some(25.0),
                    words_written: 300,
                    chapter_slug: "ch-1".to_string(),
                    sprint_goal: None,
                }
            })
            .collect();

        let stats = calculate_stats(&sessions);

        assert_eq!(stats.current_streak, 2); // today + yesterday
        assert_eq!(stats.longest_streak, 2); // the gap breaks it
    }

    #[test]
    fn stats_streak_across_month_boundary() {
        // Test streak from Jan 30 to Feb 2 (4 consecutive days across month boundary)
        let dates = vec![
            NaiveDate::from_ymd_opt(2026, 1, 30).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 2, 2).unwrap(),
        ];

        let sessions: Vec<WritingSession> = dates
            .into_iter()
            .map(|date| {
                let dt = date.and_hms_opt(10, 0, 0).unwrap().and_utc();
                WritingSession {
                    id: dt.to_rfc3339(),
                    start: dt.to_rfc3339(),
                    end: None,
                    duration_minutes: Some(25.0),
                    words_written: 200,
                    chapter_slug: "ch-1".to_string(),
                    sprint_goal: None,
                }
            })
            .collect();

        let stats = calculate_stats(&sessions);

        assert_eq!(stats.longest_streak, 4);
    }

    #[test]
    fn stats_best_day_picks_highest_word_count() {
        let sessions = vec![
            WritingSession {
                id: "2026-02-10T10:00:00Z".to_string(),
                start: "2026-02-10T10:00:00Z".to_string(),
                end: None,
                duration_minutes: Some(25.0),
                words_written: 300,
                chapter_slug: "ch-1".to_string(),
                sprint_goal: None,
            },
            WritingSession {
                id: "2026-02-10T14:00:00Z".to_string(),
                start: "2026-02-10T14:00:00Z".to_string(),
                end: None,
                duration_minutes: Some(25.0),
                words_written: 400,
                chapter_slug: "ch-1".to_string(),
                sprint_goal: None,
            },
            WritingSession {
                id: "2026-02-11T10:00:00Z".to_string(),
                start: "2026-02-11T10:00:00Z".to_string(),
                end: None,
                duration_minutes: Some(25.0),
                words_written: 500,
                chapter_slug: "ch-1".to_string(),
                sprint_goal: None,
            },
        ];

        let stats = calculate_stats(&sessions);

        // Feb 10 had 300+400=700 total, Feb 11 had 500
        assert_eq!(stats.best_day_words, 700);
        assert_eq!(stats.best_day_date, Some("2026-02-10".to_string()));
    }

    #[test]
    fn stats_via_command_with_empty_project() {
        let (_dir, path) = setup_session_test();

        let stats = get_session_stats(&path).unwrap();
        assert_eq!(stats.total_sessions, 0);
        assert_eq!(stats.total_words, 0);
    }

    #[test]
    fn stats_via_command_with_sessions() {
        let (_dir, path) = setup_session_test();

        let sessions = vec![
            WritingSession {
                id: "2026-02-10T10:00:00Z".to_string(),
                start: "2026-02-10T10:00:00Z".to_string(),
                end: Some("2026-02-10T10:30:00Z".to_string()),
                duration_minutes: Some(30.0),
                words_written: 500,
                chapter_slug: "ch-1".to_string(),
                sprint_goal: None,
            },
            WritingSession {
                id: "2026-02-11T10:00:00Z".to_string(),
                start: "2026-02-11T10:00:00Z".to_string(),
                end: Some("2026-02-11T11:00:00Z".to_string()),
                duration_minutes: Some(60.0),
                words_written: 1000,
                chapter_slug: "ch-2".to_string(),
                sprint_goal: Some(800),
            },
        ];
        write_test_sessions(&path, sessions);

        let stats = get_session_stats(&path).unwrap();
        assert_eq!(stats.total_sessions, 2);
        assert_eq!(stats.total_words, 1500);
        assert_eq!(stats.total_minutes, 90.0);
    }

    // ── YAML round-trip ─────────────────────────────────────────────

    #[test]
    fn yaml_round_trip_preserves_all_fields() {
        let (_dir, path) = setup_session_test();

        let session = WritingSession {
            id: "2026-02-14T10:30:00+00:00".to_string(),
            start: "2026-02-14T10:30:00+00:00".to_string(),
            end: Some("2026-02-14T11:00:00+00:00".to_string()),
            duration_minutes: Some(30.0),
            words_written: 847,
            chapter_slug: "chapter-1".to_string(),
            sprint_goal: Some(500),
        };

        write_test_sessions(&path, vec![session.clone()]);
        let loaded = load_sessions(&path).unwrap();

        assert_eq!(loaded.sessions.len(), 1);
        let loaded_session = &loaded.sessions[0];
        assert_eq!(loaded_session.id, session.id);
        assert_eq!(loaded_session.start, session.start);
        assert_eq!(loaded_session.end, session.end);
        assert_eq!(loaded_session.duration_minutes, session.duration_minutes);
        assert_eq!(loaded_session.words_written, session.words_written);
        assert_eq!(loaded_session.chapter_slug, session.chapter_slug);
        assert_eq!(loaded_session.sprint_goal, session.sprint_goal);
    }

    // ── load_sessions / save_sessions ───────────────────────────────

    #[test]
    fn load_sessions_returns_empty_when_no_file() {
        let (_dir, path) = setup_session_test();

        let data = load_sessions(&path).unwrap();
        assert!(data.sessions.is_empty());
    }

    #[test]
    fn save_and_load_round_trip() {
        let (_dir, path) = setup_session_test();

        let data = SessionsData {
            sessions: vec![WritingSession {
                id: "test-id".to_string(),
                start: "2026-02-14T10:00:00Z".to_string(),
                end: None,
                duration_minutes: None,
                words_written: 0,
                chapter_slug: "ch-1".to_string(),
                sprint_goal: None,
            }],
        };

        save_sessions(&path, &data).unwrap();
        let loaded = load_sessions(&path).unwrap();

        assert_eq!(loaded.sessions.len(), 1);
        assert_eq!(loaded.sessions[0].id, "test-id");
    }

    // ── Streak edge cases ───────────────────────────────────────────

    #[test]
    fn current_streak_zero_when_no_recent_sessions() {
        // Sessions only from a week ago
        let today = Utc::now().date_naive();
        let old_date = today - chrono::Duration::days(7);
        let dt = old_date.and_hms_opt(10, 0, 0).unwrap().and_utc();

        let sessions = vec![WritingSession {
            id: dt.to_rfc3339(),
            start: dt.to_rfc3339(),
            end: None,
            duration_minutes: Some(25.0),
            words_written: 300,
            chapter_slug: "ch-1".to_string(),
            sprint_goal: None,
        }];

        let stats = calculate_stats(&sessions);
        assert_eq!(stats.current_streak, 0);
        assert_eq!(stats.longest_streak, 1);
    }

    #[test]
    fn current_streak_includes_yesterday_when_no_session_today() {
        let today = Utc::now().date_naive();

        // Sessions yesterday and the day before, but not today
        let dates = vec![
            today - chrono::Duration::days(1),
            today - chrono::Duration::days(2),
        ];

        let sessions: Vec<WritingSession> = dates
            .into_iter()
            .map(|date| {
                let dt = date.and_hms_opt(10, 0, 0).unwrap().and_utc();
                WritingSession {
                    id: dt.to_rfc3339(),
                    start: dt.to_rfc3339(),
                    end: None,
                    duration_minutes: Some(25.0),
                    words_written: 300,
                    chapter_slug: "ch-1".to_string(),
                    sprint_goal: None,
                }
            })
            .collect();

        let stats = calculate_stats(&sessions);
        // Should count from yesterday backwards
        assert_eq!(stats.current_streak, 2);
    }

    #[test]
    fn multiple_sessions_same_day_count_as_one_streak_day() {
        let today = Utc::now().date_naive();
        let dt1 = today.and_hms_opt(10, 0, 0).unwrap().and_utc();
        let dt2 = today.and_hms_opt(14, 0, 0).unwrap().and_utc();

        let sessions = vec![
            WritingSession {
                id: dt1.to_rfc3339(),
                start: dt1.to_rfc3339(),
                end: None,
                duration_minutes: Some(25.0),
                words_written: 300,
                chapter_slug: "ch-1".to_string(),
                sprint_goal: None,
            },
            WritingSession {
                id: dt2.to_rfc3339(),
                start: dt2.to_rfc3339(),
                end: None,
                duration_minutes: Some(25.0),
                words_written: 400,
                chapter_slug: "ch-1".to_string(),
                sprint_goal: None,
            },
        ];

        let stats = calculate_stats(&sessions);
        assert_eq!(stats.current_streak, 1);
        assert_eq!(stats.longest_streak, 1);
    }

    // ── get_sessions: empty result from date range filter ─────────

    #[test]
    fn get_sessions_returns_empty_when_no_sessions_match_date_range() {
        let (_dir, path) = setup_session_test();

        let sessions = vec![
            WritingSession {
                id: "2026-02-10T10:00:00Z".to_string(),
                start: "2026-02-10T10:00:00Z".to_string(),
                end: Some("2026-02-10T10:30:00Z".to_string()),
                duration_minutes: Some(30.0),
                words_written: 300,
                chapter_slug: "ch-1".to_string(),
                sprint_goal: None,
            },
            WritingSession {
                id: "2026-02-14T10:00:00Z".to_string(),
                start: "2026-02-14T10:00:00Z".to_string(),
                end: Some("2026-02-14T10:30:00Z".to_string()),
                duration_minutes: Some(30.0),
                words_written: 700,
                chapter_slug: "ch-2".to_string(),
                sprint_goal: None,
            },
        ];
        write_test_sessions(&path, sessions);

        // Filter to a range with no sessions (Feb 11-13)
        let filtered = get_sessions(&path, Some("2026-02-11"), Some("2026-02-13")).unwrap();
        assert!(
            filtered.is_empty(),
            "Expected no sessions in Feb 11-13, got {}",
            filtered.len()
        );
    }

    // ── Edge cases: sessions spanning midnight ────────────────────

    #[test]
    fn session_spanning_midnight_counted_on_start_date() {
        let (_dir, path) = setup_session_test();

        // Session starts at 11:30 PM on Feb 10, ends at 12:30 AM on Feb 11
        let sessions = vec![WritingSession {
            id: "2026-02-10T23:30:00Z".to_string(),
            start: "2026-02-10T23:30:00Z".to_string(),
            end: Some("2026-02-11T00:30:00Z".to_string()),
            duration_minutes: Some(60.0),
            words_written: 500,
            chapter_slug: "ch-1".to_string(),
            sprint_goal: None,
        }];
        write_test_sessions(&path, sessions);

        // Should be found when filtering for Feb 10 (the start date)
        let filtered = get_sessions(&path, Some("2026-02-10"), Some("2026-02-10")).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].words_written, 500);

        // Should NOT be found when filtering for only Feb 11 (the end date)
        let filtered = get_sessions(&path, Some("2026-02-11"), Some("2026-02-11")).unwrap();
        assert!(
            filtered.is_empty(),
            "Session should be counted on start date, not end date"
        );
    }

    // ── Edge cases: very large word counts ────────────────────────

    #[test]
    fn stats_handle_very_large_word_counts() {
        let sessions = vec![
            WritingSession {
                id: "2026-02-10T10:00:00Z".to_string(),
                start: "2026-02-10T10:00:00Z".to_string(),
                end: Some("2026-02-10T14:00:00Z".to_string()),
                duration_minutes: Some(240.0),
                words_written: u32::MAX, // ~4.29 billion
                chapter_slug: "ch-1".to_string(),
                sprint_goal: None,
            },
            WritingSession {
                id: "2026-02-11T10:00:00Z".to_string(),
                start: "2026-02-11T10:00:00Z".to_string(),
                end: Some("2026-02-11T14:00:00Z".to_string()),
                duration_minutes: Some(240.0),
                words_written: 1000,
                chapter_slug: "ch-2".to_string(),
                sprint_goal: None,
            },
        ];

        let stats = calculate_stats(&sessions);

        // total_words is u64 so it should not overflow
        assert_eq!(stats.total_words, u32::MAX as u64 + 1000);
        assert_eq!(stats.total_sessions, 2);
        assert_eq!(stats.total_minutes, 480.0);
        assert_eq!(stats.best_day_words, u32::MAX);
    }

    // ── YAML round-trip: None optional fields ─────────────────────

    #[test]
    fn yaml_round_trip_preserves_none_fields() {
        let (_dir, path) = setup_session_test();

        let session = WritingSession {
            id: "2026-02-14T10:30:00+00:00".to_string(),
            start: "2026-02-14T10:30:00+00:00".to_string(),
            end: None,
            duration_minutes: None,
            words_written: 0,
            chapter_slug: "chapter-1".to_string(),
            sprint_goal: None,
        };

        write_test_sessions(&path, vec![session.clone()]);
        let loaded = load_sessions(&path).unwrap();

        assert_eq!(loaded.sessions.len(), 1);
        let loaded_session = &loaded.sessions[0];
        assert_eq!(loaded_session.id, session.id);
        assert!(loaded_session.end.is_none());
        assert!(loaded_session.duration_minutes.is_none());
        assert_eq!(loaded_session.words_written, 0);
        assert!(loaded_session.sprint_goal.is_none());
    }

    // ── Direct tests of streak helper functions ──────────────────

    #[test]
    fn calculate_current_streak_empty_dates() {
        let today = Utc::now().date_naive();
        assert_eq!(calculate_current_streak(&[], today), 0);
    }

    #[test]
    fn calculate_longest_streak_empty_dates() {
        assert_eq!(calculate_longest_streak(&[]), 0);
    }

    #[test]
    fn calculate_longest_streak_single_date() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 10).unwrap();
        assert_eq!(calculate_longest_streak(&[date]), 1);
    }

    #[test]
    fn calculate_longest_streak_with_gap_resets() {
        let dates = vec![
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 3).unwrap(),
            // gap
            NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 11).unwrap(),
        ];
        // First run: 3 consecutive, second run: 2 consecutive
        assert_eq!(calculate_longest_streak(&dates), 3);
    }

    #[test]
    fn calculate_longest_streak_across_year_boundary() {
        let dates = vec![
            NaiveDate::from_ymd_opt(2025, 12, 30).unwrap(),
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
        ];
        assert_eq!(calculate_longest_streak(&dates), 4);
    }

    #[test]
    fn calculate_current_streak_with_session_today() {
        let today = Utc::now().date_naive();
        let dates = vec![
            today - chrono::Duration::days(2),
            today - chrono::Duration::days(1),
            today,
        ];
        assert_eq!(calculate_current_streak(&dates, today), 3);
    }

    #[test]
    fn calculate_current_streak_grace_day_yesterday() {
        // No session today, but sessions yesterday and the day before
        let today = Utc::now().date_naive();
        let dates = vec![
            today - chrono::Duration::days(2),
            today - chrono::Duration::days(1),
        ];
        // Should count from yesterday backwards (grace period)
        assert_eq!(calculate_current_streak(&dates, today), 2);
    }

    #[test]
    fn calculate_current_streak_no_grace_after_two_days() {
        // No session today or yesterday -> streak is 0
        let today = Utc::now().date_naive();
        let dates = vec![
            today - chrono::Duration::days(3),
            today - chrono::Duration::days(2),
        ];
        assert_eq!(calculate_current_streak(&dates, today), 0);
    }

    // ── Duplicate dates in streak ─────────────────────────────────

    #[test]
    fn calculate_longest_streak_with_duplicate_dates() {
        // Multiple sessions on the same day shouldn't inflate streak
        let dates = vec![
            NaiveDate::from_ymd_opt(2026, 2, 10).unwrap(),
            NaiveDate::from_ymd_opt(2026, 2, 10).unwrap(), // duplicate
            NaiveDate::from_ymd_opt(2026, 2, 11).unwrap(),
            NaiveDate::from_ymd_opt(2026, 2, 11).unwrap(), // duplicate
        ];
        assert_eq!(calculate_longest_streak(&dates), 2);
    }

    // ── get_sessions via command with multiple sessions ──────────

    #[test]
    fn get_sessions_filters_inclusive_boundaries() {
        let (_dir, path) = setup_session_test();

        let sessions = vec![
            WritingSession {
                id: "2026-02-10T00:00:00Z".to_string(),
                start: "2026-02-10T00:00:00Z".to_string(),
                end: None,
                duration_minutes: None,
                words_written: 100,
                chapter_slug: "ch-1".to_string(),
                sprint_goal: None,
            },
            WritingSession {
                id: "2026-02-12T23:59:59Z".to_string(),
                start: "2026-02-12T23:59:59Z".to_string(),
                end: None,
                duration_minutes: None,
                words_written: 200,
                chapter_slug: "ch-2".to_string(),
                sprint_goal: None,
            },
        ];
        write_test_sessions(&path, sessions);

        // Both boundary dates should be included
        let filtered = get_sessions(&path, Some("2026-02-10"), Some("2026-02-12")).unwrap();
        assert_eq!(filtered.len(), 2);
    }

    // ── YAML round-trip: multiple sessions ────────────────────────

    #[test]
    fn yaml_round_trip_preserves_multiple_sessions() {
        let (_dir, path) = setup_session_test();

        let sessions = vec![
            WritingSession {
                id: "2026-02-10T10:00:00Z".to_string(),
                start: "2026-02-10T10:00:00Z".to_string(),
                end: Some("2026-02-10T10:30:00Z".to_string()),
                duration_minutes: Some(30.0),
                words_written: 500,
                chapter_slug: "chapter-1".to_string(),
                sprint_goal: Some(600),
            },
            WritingSession {
                id: "2026-02-11T09:00:00Z".to_string(),
                start: "2026-02-11T09:00:00Z".to_string(),
                end: None,
                duration_minutes: None,
                words_written: 0,
                chapter_slug: "chapter-2".to_string(),
                sprint_goal: None,
            },
        ];

        write_test_sessions(&path, sessions.clone());
        let loaded = load_sessions(&path).unwrap();

        assert_eq!(loaded.sessions.len(), 2);
        assert_eq!(loaded.sessions[0].id, sessions[0].id);
        assert_eq!(loaded.sessions[0].sprint_goal, Some(600));
        assert_eq!(loaded.sessions[1].id, sessions[1].id);
        assert!(loaded.sessions[1].end.is_none());
        assert!(loaded.sessions[1].sprint_goal.is_none());
    }
}
