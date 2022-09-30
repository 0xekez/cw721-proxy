use std::cmp::Ordering;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Env, StdError, StdResult, Storage};
use cw_storage_plus::Item;
use thiserror::Error;

// Need to derive ourselves instead of cw_serde as we have a custom
// partial equal implementation.
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, Copy)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Rate {
    PerBlock(u64),
    Blocks(u64),
}

pub struct RateLimiter {
    rate_limit: Item<'static, Rate>,
    last_updated_height: Item<'static, u64>,
    this_block: Item<'static, u64>,
}

#[derive(Error, Debug, PartialEq)]
pub enum RateLimitError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("rate limited. blocks until next chance: ({blocks_remaining})")]
    Limited { blocks_remaining: u64 },
}

impl RateLimiter {
    pub const fn new(
        rate_limit_key: &'static str,
        last_updated_key: &'static str,
        this_block_key: &'static str,
    ) -> Self {
        Self {
            rate_limit: Item::new(rate_limit_key),
            last_updated_height: Item::new(last_updated_key),
            this_block: Item::new(this_block_key),
        }
    }

    pub fn init(&self, storage: &mut dyn Storage, rate_limit: &Rate) -> StdResult<()> {
        self.rate_limit.save(storage, rate_limit)?;
        self.last_updated_height.save(storage, &0)
    }

    pub fn limit(&self, storage: &mut dyn Storage, env: &Env) -> Result<(), RateLimitError> {
        let last_updated = self.last_updated_height.load(storage)?;
        match self.rate_limit.load(storage)? {
            Rate::PerBlock(limit) => {
                let nfts_this_block = if last_updated == env.block.height {
                    self.this_block.load(storage)? + 1
                } else {
                    1
                };

                if nfts_this_block > limit {
                    return Err(RateLimitError::Limited {
                        blocks_remaining: 1,
                    });
                }
                self.this_block.save(storage, &nfts_this_block)?;
            }
            Rate::Blocks(min_blocks) => {
                let elapsed = env.block.height.saturating_sub(last_updated);
                if elapsed < min_blocks {
                    return Err(RateLimitError::Limited {
                        blocks_remaining: min_blocks - elapsed,
                    });
                }
            }
        }
        self.last_updated_height.save(storage, &env.block.height)?;
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
mod tests {
    use super::*;

    #[test]
    fn test_cmp() {
        assert_eq!(Rate::PerBlock(1), Rate::Blocks(1));
        assert_ne!(Rate::PerBlock(0), Rate::Blocks(0));
        assert!(Rate::PerBlock(2) > Rate::Blocks(1));
        assert!(Rate::Blocks(2) < Rate::Blocks(1));
        assert!(Rate::PerBlock(2) > Rate::PerBlock(1));
        assert!(Rate::PerBlock(2) > Rate::Blocks(1));
    }

    #[test]
    fn test_infinity() {
        let infinity = Rate::Blocks(0);
        // bitwise not. largest possible u64.
        assert!(Rate::PerBlock(!0) < infinity);
        assert!(infinity.is_infinite());
        assert!(!Rate::PerBlock(!0).is_infinite());
    }

    #[test]
    fn test_zero() {
        let zero = Rate::PerBlock(0);
        assert!(zero.is_zero());
        assert!(zero < Rate::Blocks(!0));
    }
}
