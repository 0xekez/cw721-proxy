use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Env, StdError, StdResult, Storage, Uint128};
use cw_storage_plus::Item;
use thiserror::Error;

#[cw_serde]
#[derive(Copy)]
pub enum Rate {
    PerBlock(Uint128),
    Blocks(Uint128),
}

pub struct RateLimiter {
    rate_limit: Item<'static, Rate>,
    number: Item<'static, Uint128>,
}

#[derive(Error, Debug)]
pub enum RateLimitError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("rate limited. try agan in {blocks_remaining}")]
    Limited { blocks_remaining: u64 },
}

impl RateLimiter {
    pub const fn new(rate_limit_key: &'static str, numerator_key: &'static str) -> Self {
        Self {
            rate_limit: Item::new(rate_limit_key),
            number: Item::new(numerator_key),
        }
    }

    pub fn init(&self, storage: &mut dyn Storage, rate_limit: &Rate) -> StdResult<()> {
        self.rate_limit.save(storage, rate_limit)?;
        self.number.save(
            storage,
            &match rate_limit {
                Rate::PerBlock(_) => Uint128::new(0),
                Rate::Blocks(blocks) => *blocks,
            },
        )
    }

    pub fn limit(&self, storage: &mut dyn Storage, env: &Env) -> Result<(), RateLimitError> {
        match self.rate_limit.load(storage)? {
            Rate::PerBlock(limit) => {
                let nfts_this_block = self.number.load(storage)?;
                if nfts_this_block > limit {
                    return Err(RateLimitError::Limited {
                        blocks_remaining: 1,
                    });
                }
                self.number
                    .save(storage, &(nfts_this_block + Uint128::new(1)))?
            }
            Rate::Blocks(min_blocks) => {
                let last_block = self.number.load(storage)?;
                let elapsed = Uint128::new(env.block.height.into()).saturating_sub(last_block);
                if elapsed < min_blocks {
                    return Err(RateLimitError::Limited {
                        blocks_remaining: (min_blocks - elapsed).u128() as u64,
                    });
                }
                self.number
                    .save(storage, &Uint128::new(env.block.height.into()))?
            }
        }

        Ok(())
    }
}
