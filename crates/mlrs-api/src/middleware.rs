//! HTTP middleware: rate limiting, timeouts, compression and CORS.

use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderValue, Request};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use mlrs_core::config::Settings;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};

use crate::error::ApiError;
use crate::state::AppState;

/// Sliding-window rate limiter keyed by client identifier.
#[derive(Debug)]
pub struct RateLimiter {
    max_requests: usize,
    window: Duration,
    windows: Mutex<HashMap<String, VecDeque<Instant>>>,
}

impl RateLimiter {
    pub fn new(max_requests: i64, window_seconds: i64) -> Self {
        Self {
            max_requests: max_requests.max(1) as usize,
            window: Duration::from_secs(window_seconds.max(1) as u64),
            windows: Mutex::new(HashMap::new()),
        }
    }

    pub fn check(&self, key: &str) -> (bool, f64) {
        let now = Instant::now();
        let Ok(mut windows) = self.windows.lock() else {
            return (true, 0.0);
        };
        let window = windows.entry(key.to_string()).or_default();
        while window
            .front()
            .is_some_and(|start| now.duration_since(*start) >= self.window)
        {
            window.pop_front();
        }
        if window.len() >= self.max_requests {
            let retry_after = window
                .front()
                .map(|start| self.window.saturating_sub(now.duration_since(*start)).as_secs_f64())
                .unwrap_or(0.0);
            return (false, retry_after.max(0.0));
        }
        window.push_back(now);
        (true, 0.0)
    }
}

#[derive(Clone)]
pub struct RateLimiterHandle {
    inner: Arc<RateLimiter>,
}

impl RateLimiterHandle {
    pub fn new(max_requests: i64, window_seconds: i64) -> Self {
        Self {
            inner: Arc::new(RateLimiter::new(max_requests, window_seconds)),
        }
    }

    pub fn check(&self, key: &str) -> (bool, f64) {
        self.inner.check(key)
    }
}

pub fn rate_limiter(settings: &Settings) -> RateLimiterHandle {
    RateLimiterHandle::new(
        settings.rate_limit_max_requests,
        settings.rate_limit_window_seconds,
    )
}

fn client_key(connect_info: &ConnectInfo<SocketAddr>, request: &axum::http::Request<Body>) -> String {
    if let Some(forwarded) = request.headers().get("x-forwarded-for") {
        if let Ok(value) = forwarded.to_str() {
            if let Some(first) = value.split(',').next() {
                let trimmed = first.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }
    connect_info.0.ip().to_string()
}

pub async fn rate_limit(
    state: State<AppState>,
    connect_info: ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if !state.settings.rate_limit_enabled {
        return next.run(request).await;
    }
    let key = client_key(&connect_info, &request);    let (allowed, retry_after) = state.rate_limiter.check(&key);
    if !allowed {
        let error = ApiError::new(
            429,
            "RATE_LIMITED",
            "Too many requests. Please slow down and retry later.",
        )
        .with_details(json!({ "retryAfterSeconds": (retry_after * 100.0).round() / 100.0 }));
        let mut response = error.into_response();
        response.headers_mut().insert(
            axum::http::header::RETRY_AFTER,
            HeaderValue::from_str(&((retry_after.floor() as u64) + 1).to_string())
                .unwrap_or_else(|_| HeaderValue::from_static("1")),
        );
        return response;
    }
    next.run(request).await
}

pub async fn timeout(
    state: State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let seconds = state.settings.request_timeout_seconds.max(1) as u64;
    match tokio::time::timeout(Duration::from_secs(seconds), next.run(request)).await {
        Ok(response) => response,
        Err(_) => ApiError::new(
            503,
            "REQUEST_TIMEOUT",
            "The request took too long to complete.",
        )
        .into_response(),
    }
}

pub fn compression(_settings: &Settings) -> CompressionLayer {
    CompressionLayer::new()
}

pub fn cors(settings: &Settings) -> CorsLayer {
    let layer = CorsLayer::new()
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
            axum::http::Method::HEAD,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::HeaderName::from_static("idempotency-key"),
        ]);
    let layer = if settings.cors_origins.iter().any(|origin| origin == "*") {
        layer.allow_origin(Any)
    } else {
        let origins: Vec<HeaderValue> = settings
            .cors_origins
            .iter()
            .filter_map(|origin| HeaderValue::from_str(origin).ok())
            .collect();
        layer.allow_origin(origins)
    };
    layer.allow_credentials(settings.cors_allow_credentials)
}
