use cw_ics721_governance::Governance;
use cw_rate_limiter::RateLimiter;

pub const GOVERNANCE: Governance = Governance::new();
pub const RATE_LIMIT: RateLimiter = RateLimiter::new("rate_limit", "sender");
