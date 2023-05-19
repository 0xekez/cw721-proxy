use cosmwasm_std::StdError;
use cw721_governed_proxy::error::ContractError as GovernanceContractError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    GovernanceError(#[from] GovernanceContractError),

    #[error("{requestee} not whitelisted!")]
    NotWhitelisted { requestee: String },
}
