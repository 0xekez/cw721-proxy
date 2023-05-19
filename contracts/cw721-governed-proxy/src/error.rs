use cosmwasm_std::{Coin, StdError};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("Unauthorized addr {addr}")]
    Unauthorized { addr: String },

    #[error(transparent)]
    Payment(#[from] PaymentError),

    #[error("Incorrect payment amount: {0} != {1}")]
    IncorrectPaymentAmount(Coin, Coin),

    #[error("No approval for {spender} in collection {collection}")]
    MissingApproval { spender: String, collection: String },
}
