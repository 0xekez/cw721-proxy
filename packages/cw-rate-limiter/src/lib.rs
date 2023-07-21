use std::cmp::Ordering;

use cosmwasm_schema::cw_serde;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Env, StdError, Storage};
use cw_storage_plus::{Item, Map};
use thiserror::Error;

// Need to derive ourselves instead of cw_serde as we have a custom
// partial equal implementation.
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, Copy)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Rate {
    PerBlock(u64),
    Blocks(u64),
}

#[cw_serde]
#[derive(Default)]
struct RateInfo {
    last_updated_height: u64,
    this_block: u64,
}

pub struct RateLimiter<'a, 'b> {
    rate_limit: Item<'a, Rate>,
    rates: Map<'a, &'b str, RateInfo>,
}

#[derive(Error, Debug, PartialEq)]
pub enum RateLimitError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("rate must be non-zero")]
    ZeroRate,

    #[error("rate limit reached for key ({key}). blocks until next chance: ({blocks_remaining})")]
    Limited { key: String, blocks_remaining: u64 },
}

impl<'a> RateLimiter<'a, '_> {
    pub const fn new(rate_limit_key: &'a str, rates_key: &'a str) -> Self {
        Self {
            rate_limit: Item::new(rate_limit_key),
            rates: Map::new(rates_key),
        }
    }

    pub fn init(&self, storage: &mut dyn Storage, rate_limit: &Rate) -> Result<(), RateLimitError> {
        if rate_limit.is_zero() {
            return Err(RateLimitError::ZeroRate {});
        }
        self.rate_limit.save(storage, rate_limit)?;
        Ok(())
    }

    pub fn limit(
        &self,
        storage: &mut dyn Storage,
        env: &Env,
        key: &str,
    ) -> Result<(), RateLimitError> {
        let RateInfo {
            last_updated_height,
            this_block,
        } = self.rates.may_load(storage, key)?.unwrap_or_default();
        let next_value = match self.rate_limit.load(storage)? {
            Rate::PerBlock(limit) => {
                let this_block = if last_updated_height == env.block.height {
                    this_block + 1
                } else {
                    1
                };

                if this_block > limit {
                    return Err(RateLimitError::Limited {
                        blocks_remaining: 1,
                        key: key.to_string(),
                    });
                }
                this_block
            }
            Rate::Blocks(min_blocks) => {
                let elapsed = env.block.height.saturating_sub(last_updated_height);
                if elapsed < min_blocks {
                    return Err(RateLimitError::Limited {
                        blocks_remaining: min_blocks - elapsed,
                        key: key.to_string(),
                    });
                }
                0
            }
        };
        self.rates.save(
            storage,
            key,
            &RateInfo {
                last_updated_height: env.block.height,
                this_block: next_value,
            },
        )?;
        Ok(())
    }

    pub fn query_limit(&self, storage: &dyn Storage) -> Result<Rate, StdError> {
        self.rate_limit.load(storage)
    }
}

impl Rate {
    pub fn is_zero(self) -> bool {
        match self {
            Self::Blocks(_) => false,
            Self::PerBlock(limit) => limit == 0,
        }
    }

    pub fn is_infinite(self) -> bool {
        match self {
            Self::Blocks(blocks) => blocks == 0,
            Self::PerBlock(_) => false,
        }
    }
}

impl PartialOrd for Rate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Rate {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}
impl Eq for Rate {}

impl Ord for Rate {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Rate::PerBlock(l), Rate::PerBlock(r)) => l.cmp(r),
            (Rate::PerBlock(l), Rate::Blocks(r)) => {
                if *l == 1 && l == r {
                    Ordering::Equal
                } else if *r == 0 || *l == 0 {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }
            (Rate::Blocks(_), Rate::PerBlock(_)) => other.cmp(self).reverse(),
            (Rate::Blocks(l), Rate::Blocks(r)) => r.cmp(l),
        }
    }
}

#[cfg(test)]
mod tests;
