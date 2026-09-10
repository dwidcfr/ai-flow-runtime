use std::thread;
use std::time::Duration;

use tracing::warn;

use crate::config::RetryConfig;
use crate::errors::{GeminiError, Result};

pub struct RetryPolicy {
    config: RetryConfig,
}

impl RetryPolicy {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let multiplier = 2u32.saturating_pow(attempt);
        let delay_ms = self.config.base_delay.as_millis() as u64 * multiplier as u64;
        let capped = delay_ms.min(self.config.max_delay.as_millis() as u64);
        Duration::from_millis(capped)
    }

    pub fn execute<F, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Result<T>,
    {
        let max_attempts = self.config.max_attempts.max(1);
        let mut last_error = GeminiError::Network("no attempts made".to_string());

        for attempt in 0..max_attempts {
            match operation() {
                Ok(value) => return Ok(value),
                Err(err) => {
                    let retryable = err.is_retryable();
                    last_error = err;

                    if !retryable || attempt + 1 >= max_attempts {
                        return Err(last_error);
                    }

                    let delay = self.delay_for_attempt(attempt);
                    warn!(
                        attempt = attempt + 1,
                        max_attempts,
                        delay_ms = delay.as_millis(),
                        error = %last_error,
                        "retrying Gemini request"
                    );
                    thread::sleep(delay);
                }
            }
        }

        Err(last_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    fn test_retry_config() -> RetryConfig {
        RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
        }
    }

    #[test]
    fn delay_grows_exponentially() {
        let policy = RetryPolicy::new(test_retry_config());
        assert_eq!(policy.delay_for_attempt(0).as_millis(), 1);
        assert_eq!(policy.delay_for_attempt(1).as_millis(), 2);
        assert_eq!(policy.delay_for_attempt(2).as_millis(), 4);
    }

    #[test]
    fn delay_is_capped() {
        let policy = RetryPolicy::new(RetryConfig {
            max_attempts: 5,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_millis(150),
        });
        assert_eq!(policy.delay_for_attempt(5).as_millis(), 150);
    }

    #[test]
    fn retries_on_retryable_error() {
        let policy = RetryPolicy::new(test_retry_config());
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);

        let result = policy.execute(move || {
            let n = counter_clone.fetch_add(1, Ordering::SeqCst);
            if n < 2 {
                Err(GeminiError::RateLimited)
            } else {
                Ok("ok")
            }
        });

        assert_eq!(result.unwrap(), "ok");
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn does_not_retry_unauthorized() {
        let policy = RetryPolicy::new(test_retry_config());
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);

        let result: std::result::Result<&str, GeminiError> = policy.execute(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Err(GeminiError::Unauthorized)
        });

        assert!(matches!(result, Err(GeminiError::Unauthorized)));
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
