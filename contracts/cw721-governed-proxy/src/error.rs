use cw_ics721_governance::GovernanceError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Governance(#[from] GovernanceError),
}
