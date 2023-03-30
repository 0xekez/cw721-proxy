use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("Unauthorized addr {addr}")]
    Unauthorized { addr: String },

    #[error("Unauthorized code id {code_id}")]
    UnauthorizedCodeId { code_id: u64 },
}
