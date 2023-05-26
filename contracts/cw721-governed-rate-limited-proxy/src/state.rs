use cw_rate_limiter::RateLimiter;

pub const CONTRACT_NAME: &str = "crates.io:cw721-proxy-code-id";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const RATE_LIMITER: RateLimiter = RateLimiter::new("rate_limit", "sender");
