use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("Unauthorized addr {addr}")]
    Unauthorized { addr: String },

    #[error("Unauthorized channel {channel}")]
    UnauthorizedChannel { channel: String },
}
