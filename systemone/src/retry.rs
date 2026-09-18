use std::collections::BTreeSet;
use std::time::Duration;

/// Retry configuration; see [`RetryPolicy::default`] for the defaults.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum retries after the initial attempt; `0` disables retries.
    pub max_retries: u32,
    /// First backoff delay, doubled up to `backoff_max`.
    pub backoff_initial: Duration,
    /// Maximum backoff delay.
    pub backoff_max: Duration,
    /// Fraction of each backoff delay randomly subtracted, from 0 to 1.
    pub backoff_jitter: f64,
    /// HTTP status codes to retry.
    pub retryable_statuses: BTreeSet<u16>,
    /// Honor `Retry-After` and the nonstandard `retry-after-ms`, up to `max_retry_after`.
    pub respect_retry_after: bool,
    /// Maximum server retry delay to honor; longer delays fall back to backoff.
    pub max_retry_after: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        let mut retryable_statuses: BTreeSet<u16> = (500..=599).collect();
        retryable_statuses.insert(408);
        retryable_statuses.insert(429);
        Self {
            max_retries: 2,
            backoff_initial: Duration::from_millis(500),
            backoff_max: Duration::from_secs(5),
            backoff_jitter: 0.25,
            retryable_statuses,
            respect_retry_after: true,
            max_retry_after: Duration::from_secs(60),
        }
    }
}

impl RetryPolicy {
    /// Delay before the next attempt (`attempt` is zero-based: `0` for the first retry).
    pub(crate) fn delay_for(&self, attempt: u32, retry_after: Option<Duration>) -> Duration {
        if self.respect_retry_after
            && let Some(retry_after) = retry_after.filter(|d| *d <= self.max_retry_after)
        {
            return retry_after;
        }
        let exponential = self
            .backoff_initial
            .saturating_mul(1u32.checked_shl(attempt).unwrap_or(u32::MAX))
            .min(self.backoff_max);
        exponential.mul_f64(1.0 - fastrand::f64() * self.backoff_jitter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(408, true; "request_timeout")]
    #[test_case(429, true; "rate_limited")]
    #[test_case(500, true; "server_error")]
    #[test_case(599, true; "server_error_max")]
    #[test_case(400, false; "bad_request")]
    #[test_case(404, false; "not_found")]
    fn default_retryable_statuses(status: u16, expected: bool) {
        assert_eq!(
            RetryPolicy::default().retryable_statuses.contains(&status),
            expected
        );
    }

    #[test]
    fn retry_after_wins_over_backoff_when_within_max() {
        let policy = RetryPolicy::default();
        let delay = policy.delay_for(0, Some(Duration::from_secs(3)));
        assert_eq!(delay, Duration::from_secs(3));
    }

    #[test]
    fn retry_after_beyond_max_falls_back_to_backoff() {
        let policy = RetryPolicy::default();
        let delay = policy.delay_for(0, Some(Duration::from_secs(120)));
        assert!(delay <= policy.backoff_initial);
    }

    #[test]
    fn backoff_grows_and_caps() {
        let policy = RetryPolicy::default();
        let first = policy.delay_for(0, None);
        let capped = policy.delay_for(20, None);
        assert!(first <= policy.backoff_initial);
        assert!(capped <= policy.backoff_max);
    }
}
