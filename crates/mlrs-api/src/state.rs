//! Shared application state handed to every route and middleware.

use mlrs_core::config::Settings;
use mlrs_db::Database;

use crate::middleware::RateLimiterHandle;
use crate::worker::WorkerHandle;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub settings: Settings,
    pub worker: WorkerHandle,
    pub rate_limiter: RateLimiterHandle,
}

impl AppState {
    pub fn new(db: Database, settings: Settings, worker: WorkerHandle) -> Self {
        Self {
            db,
            settings: settings.clone(),
            worker,
            rate_limiter: RateLimiterHandle::new(
                settings.rate_limit_max_requests,
                settings.rate_limit_window_seconds,
            ),
        }
    }
}
