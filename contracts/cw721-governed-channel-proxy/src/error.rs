use cosmwasm_std::StdError;
use cw_ics721_governance::GovernanceError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Governance(#[from] GovernanceError),

    #[error("{requestee} not whitelisted!")]
    NotWhitelisted { requestee: String },
}
