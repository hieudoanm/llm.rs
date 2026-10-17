//! Central configuration for the llm Rust backend.
//!
//! Every environment variable is resolved here, exactly once, at process
//! start. Modules call [`get_settings`] rather than reading the environment
//! directly. Invalid values fall back to the documented default with a single
//! warning, matching the Python service.
//!
//! The Rust port targets SQLite so it can run locally and inside Tauri. The
//! `DATABASE_URL` and `SKL_DATA_DIR` names are unchanged; their defaults are
//! relative to the working directory rather than `/var/lib/llm`.

use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

use tracing::warn;

pub const DEFAULT_DATABASE_URL: &str = "sqlite://.llm-data/llm.db";
pub const DEFAULT_DATA_DIR: &str = ".llm-data";
pub const DEFAULT_MAX_UPLOAD_MB: i64 = 10;
pub const DEFAULT_LOG_LEVEL: &str = "INFO";
pub const DEFAULT_CORS_ORIGINS: &[&str] = &["http://localhost:3000", "http://127.0.0.1:3000"];
pub const DEFAULT_CORS_ALLOW_CREDENTIALS: bool = true;
pub const DEFAULT_ENABLE_INTERACTIVE_DOCS: bool = true;
pub const DEFAULT_MAX_CONCURRENT_JOBS: i64 = 1;
pub const DEFAULT_JOB_TIMEOUT_SECONDS: i64 = 1800;
pub const DEFAULT_JOB_MAX_ATTEMPTS: i64 = 2;
pub const DEFAULT_JOB_RETRY_BASE_SECONDS: i64 = 2;
pub const DEFAULT_WORKER_STALE_SECONDS: i64 = 3600;
pub const DEFAULT_RATE_LIMIT_ENABLED: bool = true;
pub const DEFAULT_RATE_LIMIT_MAX_REQUESTS: i64 = 100;
pub const DEFAULT_RATE_LIMIT_WINDOW_SECONDS: i64 = 60;
pub const DEFAULT_COMPRESSION_ENABLED: bool = true;
pub const DEFAULT_COMPRESSION_MIN_SIZE: i64 = 500;
pub const DEFAULT_REQUEST_TIMEOUT_SECONDS: i64 = 30;

pub const MAX_UPLOAD_MB: i64 = 1024;
pub const MAX_CONCURRENT_JOBS: i64 = 16;
pub const MAX_JOB_TIMEOUT_SECONDS: i64 = 86_400;
pub const MAX_JOB_MAX_ATTEMPTS: i64 = 5;
pub const MAX_JOB_RETRY_BASE_SECONDS: i64 = 60;
pub const MAX_WORKER_STALE_SECONDS: i64 = 86_400;
pub const MAX_RATE_LIMIT_MAX_REQUESTS: i64 = 10_000;
pub const MAX_RATE_LIMIT_WINDOW_SECONDS: i64 = 3600;
pub const MAX_COMPRESSION_MIN_SIZE: i64 = 65_536;
pub const MAX_REQUEST_TIMEOUT_SECONDS: i64 = 3600;

pub const LOG_LEVELS: &[&str] = &["DEBUG", "INFO", "WARNING", "ERROR"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub database_url: String,
    pub data_dir: PathBuf,
    pub max_upload_mb: i64,
    pub log_level: String,
    pub cors_origins: Vec<String>,
    pub cors_allow_credentials: bool,
    pub enable_interactive_docs: bool,
    pub max_concurrent_jobs: i64,
    pub job_timeout_seconds: i64,
    pub job_max_attempts: i64,
    pub job_retry_base_seconds: i64,
    pub worker_stale_seconds: i64,
    pub rate_limit_enabled: bool,
    pub rate_limit_max_requests: i64,
    pub rate_limit_window_seconds: i64,
    pub compression_enabled: bool,
    pub compression_min_size: i64,
    pub request_timeout_seconds: i64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            database_url: DEFAULT_DATABASE_URL.to_string(),
            data_dir: PathBuf::from(DEFAULT_DATA_DIR),
            max_upload_mb: DEFAULT_MAX_UPLOAD_MB,
            log_level: DEFAULT_LOG_LEVEL.to_string(),
            cors_origins: DEFAULT_CORS_ORIGINS.iter().map(|s| s.to_string()).collect(),
            cors_allow_credentials: DEFAULT_CORS_ALLOW_CREDENTIALS,
            enable_interactive_docs: DEFAULT_ENABLE_INTERACTIVE_DOCS,
            max_concurrent_jobs: DEFAULT_MAX_CONCURRENT_JOBS,
            job_timeout_seconds: DEFAULT_JOB_TIMEOUT_SECONDS,
            job_max_attempts: DEFAULT_JOB_MAX_ATTEMPTS,
            job_retry_base_seconds: DEFAULT_JOB_RETRY_BASE_SECONDS,
            worker_stale_seconds: DEFAULT_WORKER_STALE_SECONDS,
            rate_limit_enabled: DEFAULT_RATE_LIMIT_ENABLED,
            rate_limit_max_requests: DEFAULT_RATE_LIMIT_MAX_REQUESTS,
            rate_limit_window_seconds: DEFAULT_RATE_LIMIT_WINDOW_SECONDS,
            compression_enabled: DEFAULT_COMPRESSION_ENABLED,
            compression_min_size: DEFAULT_COMPRESSION_MIN_SIZE,
            request_timeout_seconds: DEFAULT_REQUEST_TIMEOUT_SECONDS,
        }
    }
}

impl Settings {
    /// Explicit configuration, for embedding (e.g. from a Tauri app).
    pub fn new(database_url: impl Into<String>, data_dir: impl Into<PathBuf>) -> Self {
        Self {
            database_url: database_url.into(),
            data_dir: data_dir.into(),
            ..Self::default()
        }
    }

    pub fn max_upload_bytes(&self) -> usize {
        self.max_upload_mb.max(0) as usize * 1024 * 1024
    }

    /// Resolve the SQLite path from `database_url` when it is a file URL.
    pub fn sqlite_path(&self) -> Option<PathBuf> {
        let url = self.database_url.trim();
        let raw = url
            .strip_prefix("sqlite://")
            .or_else(|| url.strip_prefix("sqlite:"))
            .unwrap_or(url);
        if raw.is_empty() || raw == ":memory:" || raw.starts_with("file:") {
            return None;
        }
        Some(PathBuf::from(raw))
    }

    pub fn from_env() -> Self {
        let allow_credentials = bool_env(
            "SKL_CORS_ALLOW_CREDENTIALS",
            DEFAULT_CORS_ALLOW_CREDENTIALS,
        );
        Self {
            database_url: database_url(),
            data_dir: data_dir(),
            max_upload_mb: int_env(
                "SKL_MAX_UPLOAD_MB",
                DEFAULT_MAX_UPLOAD_MB,
                1,
                MAX_UPLOAD_MB,
            ),
            log_level: log_level(),
            cors_origins: cors_origins(allow_credentials),
            cors_allow_credentials: allow_credentials,
            enable_interactive_docs: bool_env(
                "SKL_ENABLE_INTERACTIVE_DOCS",
                DEFAULT_ENABLE_INTERACTIVE_DOCS,
            ),
            max_concurrent_jobs: int_env(
                "SKL_MAX_CONCURRENT_JOBS",
                DEFAULT_MAX_CONCURRENT_JOBS,
                1,
                MAX_CONCURRENT_JOBS,
            ),
            job_timeout_seconds: int_env(
                "SKL_JOB_TIMEOUT_SECONDS",
                DEFAULT_JOB_TIMEOUT_SECONDS,
                1,
                MAX_JOB_TIMEOUT_SECONDS,
            ),
            job_max_attempts: int_env(
                "SKL_JOB_MAX_ATTEMPTS",
                DEFAULT_JOB_MAX_ATTEMPTS,
                1,
                MAX_JOB_MAX_ATTEMPTS,
            ),
            job_retry_base_seconds: int_env(
                "SKL_JOB_RETRY_BASE_SECONDS",
                DEFAULT_JOB_RETRY_BASE_SECONDS,
                1,
                MAX_JOB_RETRY_BASE_SECONDS,
            ),
            worker_stale_seconds: int_env(
                "SKL_WORKER_STALE_SECONDS",
                DEFAULT_WORKER_STALE_SECONDS,
                1,
                MAX_WORKER_STALE_SECONDS,
            ),
            rate_limit_enabled: bool_env("SKL_RATE_LIMIT_ENABLED", DEFAULT_RATE_LIMIT_ENABLED),
            rate_limit_max_requests: int_env(
                "SKL_RATE_LIMIT_MAX_REQUESTS",
                DEFAULT_RATE_LIMIT_MAX_REQUESTS,
                1,
                MAX_RATE_LIMIT_MAX_REQUESTS,
            ),
            rate_limit_window_seconds: int_env(
                "SKL_RATE_LIMIT_WINDOW_SECONDS",
                DEFAULT_RATE_LIMIT_WINDOW_SECONDS,
                1,
                MAX_RATE_LIMIT_WINDOW_SECONDS,
            ),
            compression_enabled: bool_env("SKL_COMPRESSION_ENABLED", DEFAULT_COMPRESSION_ENABLED),
            compression_min_size: int_env(
                "SKL_COMPRESSION_MIN_SIZE",
                DEFAULT_COMPRESSION_MIN_SIZE,
                0,
                MAX_COMPRESSION_MIN_SIZE,
            ),
            request_timeout_seconds: int_env(
                "SKL_REQUEST_TIMEOUT_SECONDS",
                DEFAULT_REQUEST_TIMEOUT_SECONDS,
                1,
                MAX_REQUEST_TIMEOUT_SECONDS,
            ),
        }
    }
}

static SETTINGS: OnceLock<RwLock<Settings>> = OnceLock::new();

fn cell() -> &'static RwLock<Settings> {
    SETTINGS.get_or_init(|| RwLock::new(Settings::from_env()))
}

/// Return the process-wide settings, building them on first use.
pub fn get_settings() -> Settings {
    cell().read().expect("settings lock poisoned").clone()
}

/// Replace the process-wide settings (used by embedding and tests).
pub fn set_settings(settings: Settings) {
    *cell().write().expect("settings lock poisoned") = settings;
}

/// Drop the cached settings so the next call re-reads the environment.
pub fn reset_settings() {
    *cell().write().expect("settings lock poisoned") = Settings::from_env();
}

fn env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|raw| raw.trim().to_string())
        .filter(|raw| !raw.is_empty())
}

fn warn_invalid(name: &str, value: &str, reason: &str, fallback: &dyn std::fmt::Debug) {
    warn!(
        "Invalid value for {name}={value:?} ({reason}); falling back to {fallback:?}."
    );
}

fn bool_env(name: &str, default: bool) -> bool {
    let Some(raw) = env(name) else {
        return default;
    };
    match raw.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => true,
        "0" | "false" | "no" | "off" => false,
        _ => {
            warn_invalid(
                name,
                &raw,
                "expected one of 1/0/true/false/yes/no/on/off",
                &default,
            );
            default
        }
    }
}

fn int_env(name: &str, default: i64, minimum: i64, maximum: i64) -> i64 {
    let Some(raw) = env(name) else {
        return default;
    };
    let Ok(value) = raw.parse::<i64>() else {
        warn_invalid(name, &raw, "not an integer", &default);
        return default;
    };
    if !(minimum..=maximum).contains(&value) {
        warn_invalid(
            name,
            &raw,
            &format!("outside the allowed range {minimum}..{maximum}"),
            &default,
        );
        return default;
    }
    value
}

fn list_env(name: &str, default: &[&str]) -> Vec<String> {
    match env(name) {
        None => default.iter().map(|s| s.to_string()).collect(),
        Some(raw) => raw
            .split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(|item| item.to_string())
            .collect(),
    }
}

fn normalise_origin(entry: &str) -> Option<String> {
    if entry.ends_with('/') {
        return None;
    }
    let (scheme, rest) = entry.split_once("://")?;
    if scheme != "http" && scheme != "https" {
        return None;
    }
    if rest.is_empty()
        || rest.contains('/')
        || rest.contains('?')
        || rest.contains('#')
        || rest.contains('@')
    {
        return None;
    }
    Some(format!("{scheme}://{rest}"))
}

fn cors_origins(allow_credentials: bool) -> Vec<String> {
    let mut origins = Vec::new();
    for entry in list_env("SKL_CORS_ORIGINS", DEFAULT_CORS_ORIGINS) {
        if entry == "*" {
            if allow_credentials {
                warn_invalid(
                    "SKL_CORS_ORIGINS",
                    "*",
                    "a wildcard origin is not allowed while credentials are enabled",
                    &"the wildcard was dropped",
                );
                continue;
            }
            origins.push("*".to_string());
            continue;
        }
        match normalise_origin(&entry) {
            Some(origin) => origins.push(origin),
            None => warn_invalid(
                "SKL_CORS_ORIGINS",
                &entry,
                "not a valid origin",
                &"the entry was dropped",
            ),
        }
    }
    origins
}

fn database_url() -> String {
    env("DATABASE_URL").unwrap_or_else(|| DEFAULT_DATABASE_URL.to_string())
}

fn data_dir() -> PathBuf {
    let Some(raw) = env("SKL_DATA_DIR") else {
        return PathBuf::from(DEFAULT_DATA_DIR);
    };
    let path = PathBuf::from(&raw);
    if path.is_absolute() {
        path
    } else {
        // Relative directories are valid for the local/Tauri build; unlike the
        // container build they are resolved against the current directory.
        path
    }
}

fn log_level() -> String {
    let Some(raw) = env("SKL_LOG_LEVEL") else {
        return DEFAULT_LOG_LEVEL.to_string();
    };
    let upper = raw.to_ascii_uppercase();
    if LOG_LEVELS.contains(&upper.as_str()) {
        upper
    } else {
        warn_invalid(
            "SKL_LOG_LEVEL",
            &raw,
            &format!("expected one of {}", LOG_LEVELS.join("/")),
            &DEFAULT_LOG_LEVEL,
        );
        DEFAULT_LOG_LEVEL.to_string()
    }
}

/// Set up the global tracing subscriber using the configured log level.
pub fn setup_logging(level: &str) {
    let filter = std::env::var("RUST_LOG")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| level.to_ascii_lowercase());
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(filter))
        .with_target(false)
        .try_init();
}
