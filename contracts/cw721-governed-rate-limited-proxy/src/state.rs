use cw721_governed_proxy::state::Cw721GovernanceProxy;
use cw_rate_limiter::RateLimiter;

pub const CONTRACT_NAME: &str = "crates.io:cw721-proxy-code-id";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Cw721GovernedChannelProxy<'a, 'b> {
    pub governance: Cw721GovernanceProxy<'a>,
    pub rate_limit: RateLimiter<'a, 'b>,
}

impl Default for Cw721GovernedChannelProxy<'_, '_> {
    fn default() -> Self {
        Self {
            governance: Default::default(),
            rate_limit: RateLimiter::new("rate_limit", "sender"),
        }
    }
}
