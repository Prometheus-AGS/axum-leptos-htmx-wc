use crate::AppState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use std::num::NonZeroU32;
use std::sync::Arc;
use tracing::warn;

/// Wrapper around Governor Rate Limiter to be stored in AppState
/// We use a generic non-keyed limiter for global rate limiting as per current design.
/// (Keyed by IP would require extracting IP which is added complexity).
#[derive(Debug, Clone)]
pub struct AppRateLimiter {
    limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl AppRateLimiter {
    pub fn new(requests_per_second: f32, burst_size: u32) -> Self {
        // Convert f32 rate to Quota. Per second.
        // Governor requires non-zero.
        let burst = NonZeroU32::new(burst_size).unwrap_or(NonZeroU32::new(1).unwrap());

        // Convert requests_per_second to duration between requests approx?
        // Governor's Quota::per_second takes a u32.
        // If we want fractional, we might need per_period.
        // Let's assume u32 for now or ceil.
        let rps = NonZeroU32::new(requests_per_second.ceil() as u32)
            .unwrap_or(NonZeroU32::new(1).unwrap());

        let quota = Quota::per_second(rps).allow_burst(burst);

        Self {
            limiter: Arc::new(RateLimiter::direct(quota)),
        }
    }

    pub fn check(&self) -> bool {
        self.limiter.check().is_ok()
    }
}

/// Middleware to enforce rate limits
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if state.config.resilience.rate_limit_enabled {
        if !state.rate_limiter.check() {
            warn!("Rate limit exceeded");
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
    }
    Ok(next.run(req).await)
}
