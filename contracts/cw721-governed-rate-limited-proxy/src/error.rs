use cosmwasm_std::StdError;
use cw_ics721_governance::GovernanceError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Governance(#[from] GovernanceError),

    #[error(transparent)]
    Rate(#[from] cw_rate_limiter::RateLimitError),

    #[error("rate must be non-zero")]
    ZeroRate {},
}
