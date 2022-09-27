use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Rate(#[from] cw_rate_limiter::RateLimitError),

    #[error("rate must be non-zero")]
    ZeroRate {},
}
