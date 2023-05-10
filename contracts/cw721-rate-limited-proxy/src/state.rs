use cosmwasm_std::Addr;
use cw_storage_plus::Item;

use cw_rate_limiter::RateLimiter;

pub const OWNER: Item<Addr> = Item::new("owner");
pub const RATE_LIMIT: RateLimiter = RateLimiter::new("rate_limit", "sender");
pub const ORIGIN: Item<Addr> = Item::new("origin");
