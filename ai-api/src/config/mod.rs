use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub data_root: PathBuf,
    pub flows_dir: PathBuf,
    pub request_timeout: Duration,
    pub max_body_size: usize,
    pub log_level: String,
    pub include_router_decision: bool,
    pub log_user_messages: bool,
    pub version: String,
    pub default_modules: Vec<DefaultModuleConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DefaultModuleConfig {
    pub module_id: String,
    pub path: String,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid configuration: {0}")]
    Invalid(String),
}

impl ApiConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let data_root = env_path("API_DATA_ROOT")
            .unwrap_or_else(default_data_root);
        let flows_dir = env_path("API_FLOWS_DIR")
            .unwrap_or_else(|| data_root.join("flows"));

        let request_timeout_secs = env_u64("API_REQUEST_TIMEOUT_SECS", 30);
        let max_body_size = parse_body_size(
            &std::env::var("API_MAX_BODY_SIZE").unwrap_or_else(|_| "1MB".into()),
        )?;

        let default_modules = parse_default_modules(
            std::env::var("API_DEFAULT_MODULES").ok().as_deref(),
        )?;

        Ok(Self {
            host: std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env_u16("API_PORT", 8080),
            data_root,
            flows_dir,
            request_timeout: Duration::from_secs(request_timeout_secs),
            max_body_size,
            log_level: std::env::var("API_LOG_LEVEL").unwrap_or_else(|_| "info".into()),
            include_router_decision: env_bool("API_INCLUDE_ROUTER_DECISION", false),
            log_user_messages: env_bool("API_LOG_USER_MESSAGES", false),
            version: std::env::var("API_VERSION").unwrap_or_else(|_| "0.1.0".into()),
            default_modules,
        })
    }

    pub fn for_test(data_root: impl AsRef<Path>) -> Self {
        let data_root = data_root.as_ref().to_path_buf();
        Self {
            host: "127.0.0.1".into(),
            port: 0,
            flows_dir: data_root.join("flows"),
            data_root,
            request_timeout: Duration::from_secs(30),
            max_body_size: 1024 * 1024,
            log_level: "warn".into(),
            include_router_decision: false,
            log_user_messages: false,
            version: "test".into(),
            default_modules: vec![
                DefaultModuleConfig {
                    module_id: "company".into(),
                    path: "company/acme.yaml".into(),
                },
                DefaultModuleConfig {
                    module_id: "smalltalk".into(),
                    path: "modules/smalltalk.yaml".into(),
                },
            ],
        }
    }

    pub fn resolve_data_path(&self, relative: &str) -> PathBuf {
        let path = Path::new(relative);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.data_root.join(relative)
        }
    }

    pub fn socket_addr(&self) -> Result<std::net::SocketAddr, ConfigError> {
        format!("{}:{}", self.host, self.port)
            .parse()
            .map_err(|e| ConfigError::Invalid(format!("invalid bind address: {e}")))
    }
}

fn default_data_root() -> PathBuf {
    PathBuf::from("ai-flow-runtime/data")
}

fn env_path(key: &str) -> Option<PathBuf> {
    std::env::var(key).ok().map(PathBuf::from)
}

fn env_u16(key: &str, default: u16) -> u16 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_bool(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(default)
}

fn parse_body_size(value: &str) -> Result<usize, ConfigError> {
    let upper = value.trim().to_uppercase();
    if let Some(num) = upper.strip_suffix("MB") {
        let n: usize = num
            .trim()
            .parse()
            .map_err(|_| ConfigError::Invalid(format!("invalid body size: {value}")))?;
        return Ok(n * 1024 * 1024);
    }
    if let Some(num) = upper.strip_suffix("KB") {
        let n: usize = num
            .trim()
            .parse()
            .map_err(|_| ConfigError::Invalid(format!("invalid body size: {value}")))?;
        return Ok(n * 1024);
    }
    value
        .parse()
        .map_err(|_| ConfigError::Invalid(format!("invalid body size: {value}")))
}

fn parse_default_modules(raw: Option<&str>) -> Result<Vec<DefaultModuleConfig>, ConfigError> {
    let Some(raw) = raw else {
        return Ok(Vec::new());
    };
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_str(raw)
        .map_err(|e| ConfigError::Invalid(format!("API_DEFAULT_MODULES: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_relative_path_against_data_root() {
        let config = ApiConfig::for_test("/data");
        assert_eq!(
            config.resolve_data_path("clients/sample.json"),
            PathBuf::from("/data/clients/sample.json")
        );
    }

    #[test]
    fn keeps_absolute_path() {
        let config = ApiConfig::for_test("/data");
        assert_eq!(
            config.resolve_data_path("/abs/client.json"),
            PathBuf::from("/abs/client.json")
        );
    }

    #[test]
    fn parses_body_size_mb() {
        assert_eq!(parse_body_size("2MB").unwrap(), 2 * 1024 * 1024);
    }
}
