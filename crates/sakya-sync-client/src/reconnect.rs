//! Reconnection policy with exponential backoff and jitter.
//!
//! Provides [`ReconnectPolicy`] for managing reconnection attempts with
//! configurable exponential backoff, a maximum delay cap, and random jitter
//! to prevent thundering herd problems.
//!
//! # Default Behavior
//!
//! - Base delay: 1 second
//! - Maximum delay: 60 seconds
//! - Jitter factor: 0.25 (±25%)
//! - Progression: 1s, 2s, 4s, 8s, 16s, 32s, 60s, 60s, ...
//!
//! # Example
//!
//! ```
//! use sakya_sync_client::reconnect::ReconnectPolicy;
//!
//! let mut policy = ReconnectPolicy::new();
//! let delay = policy.next_delay();
//! // delay is approximately 1s (±25% jitter)
//!
//! // After a successful reconnection, reset the counter
//! policy.reset();
//! ```

use std::time::Duration;

use rand::Rng;

/// Manages reconnection timing with exponential backoff and jitter.
///
/// Each call to [`next_delay`](Self::next_delay) returns an increasing delay
/// computed as `base_delay * 2^attempt`, capped at `max_delay`, with random
/// jitter applied within `±jitter_factor` of the computed value.
#[derive(Debug, Clone)]
pub struct ReconnectPolicy {
    /// The starting delay for the first reconnection attempt.
    base_delay: Duration,
    /// The upper bound on the computed delay (before jitter).
    max_delay: Duration,
    /// Fraction of the delay to vary randomly (e.g., 0.25 = ±25%).
    jitter_factor: f64,
    /// Number of attempts made so far.
    current_attempt: u32,
}

impl Default for ReconnectPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl ReconnectPolicy {
    /// Creates a new `ReconnectPolicy` with default settings.
    ///
    /// Defaults:
    /// - `base_delay`: 1 second
    /// - `max_delay`: 60 seconds
    /// - `jitter_factor`: 0.25 (±25%)
    pub fn new() -> Self {
        Self {
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            jitter_factor: 0.25,
            current_attempt: 0,
        }
    }

    /// Creates a new `ReconnectPolicy` with custom configuration.
    ///
    /// # Panics
    ///
    /// Panics if `jitter_factor` is negative or greater than 1.0,
    /// or if `base_delay` is greater than `max_delay`.
    pub fn with_config(base_delay: Duration, max_delay: Duration, jitter_factor: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&jitter_factor),
            "jitter_factor must be between 0.0 and 1.0, got {jitter_factor}"
        );
        assert!(
            base_delay <= max_delay,
            "base_delay ({base_delay:?}) must not exceed max_delay ({max_delay:?})"
        );

        Self {
            base_delay,
            max_delay,
            jitter_factor,
            current_attempt: 0,
        }
    }

    /// Computes the next reconnection delay and increments the attempt counter.
    ///
    /// The delay is computed as:
    /// 1. `raw = base_delay * 2^current_attempt`
    /// 2. `capped = min(raw, max_delay)`
    /// 3. `jittered = capped * uniform_random(1 - jitter_factor, 1 + jitter_factor)`
    ///
    /// The attempt counter is incremented after computing the delay.
    pub fn next_delay(&mut self) -> Duration {
        let base_ms = self.base_delay.as_millis() as f64;

        // Exponential growth: base * 2^attempt
        // Use saturating_pow to avoid overflow, then cap at max_delay
        let multiplier = 2_u64.saturating_pow(self.current_attempt);
        let raw_ms = base_ms * multiplier as f64;

        // Cap at max_delay
        let max_ms = self.max_delay.as_millis() as f64;
        let capped_ms = raw_ms.min(max_ms);

        // Apply jitter: multiply by a random factor in [1 - jitter, 1 + jitter]
        let jittered_ms = if self.jitter_factor > 0.0 {
            let mut rng = rand::thread_rng();
            let jitter_low = 1.0 - self.jitter_factor;
            let jitter_high = 1.0 + self.jitter_factor;
            let jitter_multiplier = rng.gen_range(jitter_low..=jitter_high);
            capped_ms * jitter_multiplier
        } else {
            capped_ms
        };

        // Increment attempt counter (saturating to avoid overflow)
        self.current_attempt = self.current_attempt.saturating_add(1);

        Duration::from_millis(jittered_ms.round() as u64)
    }

    /// Resets the attempt counter to zero.
    ///
    /// Call this after a successful reconnection so the next failure
    /// starts from the base delay again.
    pub fn reset(&mut self) {
        self.current_attempt = 0;
    }

    /// Returns the number of attempts made so far.
    pub fn attempt_count(&self) -> u32 {
        self.current_attempt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a zero-jitter policy for deterministic testing.
    fn deterministic_policy() -> ReconnectPolicy {
        ReconnectPolicy::with_config(
            Duration::from_secs(1),
            Duration::from_secs(60),
            0.0, // no jitter
        )
    }

    #[test]
    fn exponential_growth() {
        let mut policy = deterministic_policy();

        let d0 = policy.next_delay();
        let d1 = policy.next_delay();
        let d2 = policy.next_delay();
        let d3 = policy.next_delay();

        // With zero jitter: 1s, 2s, 4s, 8s
        assert_eq!(d0, Duration::from_secs(1), "attempt 0 should be 1s");
        assert_eq!(d1, Duration::from_secs(2), "attempt 1 should be 2s");
        assert_eq!(d2, Duration::from_secs(4), "attempt 2 should be 4s");
        assert_eq!(d3, Duration::from_secs(8), "attempt 3 should be 8s");
    }

    #[test]
    fn max_cap() {
        let mut policy =
            ReconnectPolicy::with_config(Duration::from_secs(1), Duration::from_secs(10), 0.0);

        // 1, 2, 4, 8, 10(cap), 10(cap)
        let _ = policy.next_delay(); // 1
        let _ = policy.next_delay(); // 2
        let _ = policy.next_delay(); // 4
        let _ = policy.next_delay(); // 8
        let d4 = policy.next_delay(); // 16 -> capped to 10
        let d5 = policy.next_delay(); // 32 -> capped to 10

        assert_eq!(
            d4,
            Duration::from_secs(10),
            "attempt 4 should be capped at 10s"
        );
        assert_eq!(
            d5,
            Duration::from_secs(10),
            "attempt 5 should be capped at 10s"
        );
    }

    #[test]
    fn jitter_within_bounds() {
        // Use 0.25 jitter (default) with a known base and max
        let base = Duration::from_millis(1000);
        let max = Duration::from_secs(60);
        let jitter = 0.25;

        // Run many iterations to statistically verify jitter bounds
        for attempt in 0..6 {
            let expected_base_ms = 1000.0 * 2_f64.powi(attempt as i32);
            let expected_capped_ms = expected_base_ms.min(60_000.0);
            let lower_bound_ms = expected_capped_ms * (1.0 - jitter);
            let upper_bound_ms = expected_capped_ms * (1.0 + jitter);

            for _ in 0..100 {
                let mut policy = ReconnectPolicy::with_config(base, max, jitter);
                // Advance to the correct attempt
                for _ in 0..attempt {
                    let _ = policy.next_delay();
                }
                let delay = policy.next_delay();
                let delay_ms = delay.as_millis() as f64;

                assert!(
                    delay_ms >= lower_bound_ms - 1.0 && delay_ms <= upper_bound_ms + 1.0,
                    "attempt {attempt}: delay {delay_ms}ms not in [{lower_bound_ms}, {upper_bound_ms}]"
                );
            }
        }
    }

    #[test]
    fn reset_on_success() {
        let mut policy = deterministic_policy();

        // Advance several attempts
        let _ = policy.next_delay(); // 1s, attempt becomes 1
        let _ = policy.next_delay(); // 2s, attempt becomes 2
        let _ = policy.next_delay(); // 4s, attempt becomes 3
        assert_eq!(policy.attempt_count(), 3);

        // Reset simulates a successful reconnection
        policy.reset();
        assert_eq!(policy.attempt_count(), 0);

        // Next delay should be back to base
        let d = policy.next_delay();
        assert_eq!(
            d,
            Duration::from_secs(1),
            "after reset, delay should be base (1s)"
        );
        assert_eq!(policy.attempt_count(), 1);
    }

    #[test]
    fn attempt_count_tracking() {
        let mut policy = deterministic_policy();

        assert_eq!(
            policy.attempt_count(),
            0,
            "initial attempt count should be 0"
        );

        let _ = policy.next_delay();
        assert_eq!(
            policy.attempt_count(),
            1,
            "after 1 call, attempt should be 1"
        );

        let _ = policy.next_delay();
        assert_eq!(
            policy.attempt_count(),
            2,
            "after 2 calls, attempt should be 2"
        );

        let _ = policy.next_delay();
        assert_eq!(
            policy.attempt_count(),
            3,
            "after 3 calls, attempt should be 3"
        );

        policy.reset();
        assert_eq!(
            policy.attempt_count(),
            0,
            "after reset, attempt should be 0"
        );

        let _ = policy.next_delay();
        assert_eq!(
            policy.attempt_count(),
            1,
            "after reset + 1 call, attempt should be 1"
        );
    }

    #[test]
    fn default_trait_matches_new() {
        let from_new = ReconnectPolicy::new();
        let from_default = ReconnectPolicy::default();

        // Both should produce the same configuration
        assert_eq!(from_new.base_delay, from_default.base_delay);
        assert_eq!(from_new.max_delay, from_default.max_delay);
        assert!((from_new.jitter_factor - from_default.jitter_factor).abs() < f64::EPSILON);
        assert_eq!(from_new.current_attempt, from_default.current_attempt);
    }

    #[test]
    #[should_panic(expected = "jitter_factor must be between 0.0 and 1.0")]
    fn invalid_jitter_factor_panics() {
        ReconnectPolicy::with_config(Duration::from_secs(1), Duration::from_secs(60), 1.5);
    }

    #[test]
    #[should_panic(expected = "base_delay")]
    fn base_exceeds_max_panics() {
        ReconnectPolicy::with_config(Duration::from_secs(120), Duration::from_secs(60), 0.25);
    }

    #[test]
    fn high_attempt_count_does_not_overflow() {
        let mut policy =
            ReconnectPolicy::with_config(Duration::from_secs(1), Duration::from_secs(60), 0.0);

        // Simulate many attempts — should saturate at max, not panic
        for _ in 0..100 {
            let d = policy.next_delay();
            assert!(
                d <= Duration::from_secs(60),
                "delay should never exceed max"
            );
        }
        assert_eq!(policy.attempt_count(), 100);
    }

    #[test]
    fn jitter_produces_variation() {
        // With jitter enabled, multiple calls at the same attempt should not all
        // return the exact same value (probabilistically ~impossible with 0.25 jitter).
        let base = Duration::from_secs(1);
        let max = Duration::from_secs(60);

        let mut values = std::collections::HashSet::new();
        for _ in 0..50 {
            let mut policy = ReconnectPolicy::with_config(base, max, 0.25);
            let d = policy.next_delay();
            values.insert(d.as_millis());
        }

        // With 50 trials and ±25% jitter on 1000ms (range: 750-1250ms),
        // we should see at least a few distinct values
        assert!(
            values.len() > 1,
            "jitter should produce variation, but got only {} distinct values: {:?}",
            values.len(),
            values
        );
    }

    #[test]
    fn custom_base_delay() {
        let mut policy =
            ReconnectPolicy::with_config(Duration::from_millis(500), Duration::from_secs(30), 0.0);

        let d0 = policy.next_delay();
        let d1 = policy.next_delay();
        let d2 = policy.next_delay();

        assert_eq!(d0, Duration::from_millis(500));
        assert_eq!(d1, Duration::from_millis(1000));
        assert_eq!(d2, Duration::from_millis(2000));
    }

    #[test]
    fn zero_jitter_is_deterministic() {
        let mut p1 = deterministic_policy();
        let mut p2 = deterministic_policy();

        for _ in 0..10 {
            assert_eq!(p1.next_delay(), p2.next_delay());
        }
    }
}
