pub mod error;
mod execute;
pub mod instantiate;
pub mod msg;
mod query;
pub mod state;

use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};

#[cfg(test)]
pub mod tests;

// Version info for migration
pub const CONTRACT_NAME: &str = "crates.io:cw-ics721-governance";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod entry {
    use crate::{
        error::ContractError,
        msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
        state::Cw721GovernanceProxy,
    };

    use super::*;

    #[cfg(not(feature = "library"))]
    use cosmwasm_std::entry_point;
    use cosmwasm_std::Deps;
    use cw2::set_contract_version;

    // This makes a conscious choice on the various generics used by the contract
    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> StdResult<Response> {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        let governance = Cw721GovernanceProxy::new();
        governance.instantiate(deps, env, info, msg)
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        let governance = Cw721GovernanceProxy::new();
        governance.execute(deps, env, info, msg)
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        let governance = Cw721GovernanceProxy::new();
        governance.query(deps, env, msg)
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> StdResult<Response> {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        Cw721GovernanceProxy::default().migrate(deps, env, msg)
    }
}
