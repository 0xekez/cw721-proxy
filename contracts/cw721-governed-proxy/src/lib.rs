pub mod error;
pub mod msg;
pub mod state;

#[cfg(test)]
mod tests;

pub mod entry {
    use crate::{
        error::ContractError,
        msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
        state::{CONTRACT_NAME, CONTRACT_VERSION},
    };

    #[cfg(not(feature = "library"))]
    use cosmwasm_std::entry_point;
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
    use cw2::set_contract_version;

    // This makes a conscious choice on the various generics used by the contract
    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn instantiate(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> StdResult<Response> {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        cw_ics721_governance::instantiate(deps, info, msg.owner, msg.origin, msg.transfer_fee)
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        match msg {
            ExecuteMsg::Governance(action) => {
                Ok(cw_ics721_governance::execute(deps, env, &info, action)?)
            }
            ExecuteMsg::ReceiveNft(msg) => {
                Ok(cw_ics721_governance::execute_receive_nft(deps, info, msg)?)
            }
        }
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::Governance() => cw_ics721_governance::query_governance(deps.storage),
        }
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> StdResult<Response> {
        match msg {
            MigrateMsg::WithUpdate {
                origin,
                owner,
                transfer_fee,
            } => {
                // Set contract to version to latest
                set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
                Ok(cw_ics721_governance::migrate(
                    deps,
                    owner,
                    origin,
                    transfer_fee,
                )?)
            }
        }
    }
}
