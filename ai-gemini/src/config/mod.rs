use std::time::Duration;

use crate::errors::{GeminiError, Result};
use crate::types::GEMINI_API_BASE;

pub const DEFAULT_MODEL: &str = "gemini-2.5-flash";
pub const DEFAULT_TEMPERATURE: f32 = 0.2;
pub const DEFAULT_MAX_OUTPUT_TOKENS: u32 = 1024;
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;
pub const DEFAULT_RETRY_COUNT: u32 = 3;
pub const DEFAULT_RETRY_BASE_DELAY_MS: u64 = 500;
pub const DEFAULT_RETRY_MAX_DELAY_MS: u64 = 8000;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: DEFAULT_RETRY_COUNT,
            base_delay: Duration::from_millis(DEFAULT_RETRY_BASE_DELAY_MS),
            max_delay: Duration::from_millis(DEFAULT_RETRY_MAX_DELAY_MS),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub log_prompts: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self { log_prompts: false }
    }
}

#[derive(Debug, Clone)]
pub struct GeminiConfig {
    pub api_key: String,
    pub api_base: String,
    pub model_router: String,
    pub model_response: String,
    pub temperature: f32,
    pub max_output_tokens: u32,
    pub timeout: Duration,
    pub retry: RetryConfig,
    pub logging: LoggingConfig,
}

impl GeminiConfig {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            api_base: GEMINI_API_BASE.to_string(),
            model_router: DEFAULT_MODEL.to_string(),
            model_response: DEFAULT_MODEL.to_string(),
            temperature: DEFAULT_TEMPERATURE,
            max_output_tokens: DEFAULT_MAX_OUTPUT_TOKENS,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            retry: RetryConfig::default(),
            logging: LoggingConfig::default(),
        }
    }

    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("GEMINI_API_KEY").map_err(|_| {
            GeminiError::Configuration("GEMINI_API_KEY is not set".to_string())
        })?;
        if api_key.trim().is_empty() {
            return Err(GeminiError::Configuration(
                "GEMINI_API_KEY is empty".to_string(),
            ));
        }

        let mut config = Self::new(api_key);
        config.model_router = env_or_default("GEMINI_MODEL_ROUTER", DEFAULT_MODEL);
        config.model_response = env_or_default("GEMINI_MODEL_RESPONSE", DEFAULT_MODEL);
        config.temperature = env_parse("GEMINI_TEMPERATURE", DEFAULT_TEMPERATURE)?;
        config.max_output_tokens = env_parse("GEMINI_MAX_OUTPUT_TOKENS", DEFAULT_MAX_OUTPUT_TOKENS)?;
        config.timeout = Duration::from_secs(env_parse("GEMINI_TIMEOUT_SECS", DEFAULT_TIMEOUT_SECS)?);
        config.retry.max_attempts = env_parse("GEMINI_RETRY_COUNT", DEFAULT_RETRY_COUNT)?;
        config.retry.base_delay = Duration::from_millis(env_parse(
            "GEMINI_RETRY_BASE_DELAY_MS",
            DEFAULT_RETRY_BASE_DELAY_MS,
        )?);
        config.retry.max_delay = Duration::from_millis(env_parse(
            "GEMINI_RETRY_MAX_DELAY_MS",
            DEFAULT_RETRY_MAX_DELAY_MS,
        )?);
        config.logging.log_prompts = env_parse_bool("GEMINI_LOG_PROMPTS", false)?;

        Ok(config)
    }
}

fn env_or_default(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_parse<T>(key: &str, default: T) -> Result<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    match std::env::var(key) {
        Ok(value) => value.parse().map_err(|e| {
            GeminiError::Configuration(format!("invalid value for {key}: {e}"))
        }),
        Err(_) => Ok(default),
    }
}

fn env_parse_bool(key: &str, default: bool) -> Result<bool> {
    match std::env::var(key) {
        Ok(value) => match value.to_lowercase().as_str() {
            "1" | "true" | "yes" => Ok(true),
            "0" | "false" | "no" => Ok(false),
            other => Err(GeminiError::Configuration(format!(
                "invalid boolean for {key}: {other}"
            ))),
        },
        Err(_) => Ok(default),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_config_has_defaults() {
        let config = GeminiConfig::new("test-key");
        assert_eq!(config.model_router, DEFAULT_MODEL);
        assert_eq!(config.temperature, DEFAULT_TEMPERATURE);
        assert_eq!(config.retry.max_attempts, DEFAULT_RETRY_COUNT);
        assert!(!config.logging.log_prompts);
    }
}
