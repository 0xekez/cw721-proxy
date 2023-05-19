use cosmwasm_std::StdError;
use cw721_governed_proxy::error::ContractError as GovernanceContractError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    GovernanceError(#[from] GovernanceContractError),

    #[error(transparent)]
    Rate(#[from] cw_rate_limiter::RateLimitError),

    #[error("rate must be non-zero")]
    ZeroRate {},
}
