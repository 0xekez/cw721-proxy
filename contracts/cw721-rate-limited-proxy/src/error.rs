use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("Unauthorized addr {addr}")]
    Unauthorized { addr: String },

    #[error(transparent)]
    Rate(#[from] cw_rate_limiter::RateLimitError),

    #[error("rate must be non-zero")]
    ZeroRate {},
}
