use crate::AppState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Simple Token Bucket Rate Limiter
/// Not keyed by IP in this global version (IP extraction requires ConnectInfo),
/// so we just limit global requests/sec for simplicity or use a single global bucket.
///
/// Requirement: 5 req/s.
pub struct SimpleRateLimiter {
    // Last check time and tokens count
    // (last_update, tokens)
    state: Mutex<(Instant, f32)>,
    rate_per_sec: f32,
    burst_size: f32,
}

impl SimpleRateLimiter {
    pub fn new(rate_per_sec: f32, burst_size: f32) -> Self {
        Self {
            state: Mutex::new((Instant::now(), burst_size)),
            rate_per_sec,
            burst_size,
        }
    }

    pub fn check(&self) -> bool {
        let mut guard = self.state.lock().unwrap();
        let (last_update, tokens) = *guard;
        let now = Instant::now();
        let elapsed = now.duration_since(last_update).as_secs_f32();

        let mut new_tokens = tokens + (elapsed * self.rate_per_sec);
        if new_tokens > self.burst_size {
            new_tokens = self.burst_size;
        }

        if new_tokens >= 1.0 {
            *guard = (now, new_tokens - 1.0);
            true
        } else {
            // Update time even if failed? No, standard algorithm updates time
            // but we can just update tokens up to now.
            // Actually, we must update state to reflect time passage even if we deny.
            *guard = (now, new_tokens);
            false
        }
    }
}

/// Middleware to enforce rate limits
/// Middleware to enforce rate limits
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if state.config.resilience.rate_limit_enabled && !state.rate_limiter.check() {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_rate_limiter() {
        let limiter = SimpleRateLimiter::new(2.0, 5.0); // 2 req/s, 5 burst

        // Consume all burst
        assert!(limiter.check()); // 4.0
        assert!(limiter.check()); // 3.0
        assert!(limiter.check()); // 2.0
        assert!(limiter.check()); // 1.0
        assert!(limiter.check()); // 0.0

        // Next should fail (immediate)
        assert!(!limiter.check());

        // Wait for 0.6s -> +1.2 tokens -> 1.2 total -> check consumes 1 -> 0.2 left -> success
        std::thread::sleep(Duration::from_millis(600));
        assert!(limiter.check());

        // Immediate fail
        assert!(!limiter.check());
    }
}
