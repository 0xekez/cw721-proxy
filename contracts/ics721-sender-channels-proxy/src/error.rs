use cosmwasm_std::StdError;
use cw_ics721_governance::ContractError as GovernanceContractError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    GovernanceError(#[from] GovernanceContractError),

    #[error("Unauthorized channel {channel}")]
    UnauthorizedChannel { channel: String },
}
