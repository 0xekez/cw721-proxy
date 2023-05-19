pub mod error;
pub mod execute;
pub mod instantiate;
pub mod msg;
pub mod query;
pub mod state;

#[cfg(test)]
mod tests;

#[cfg(not(feature = "library"))]
pub mod entry {
    use crate::error::ContractError;
    use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
    use crate::state::{Cw721GovernedCollectionProxy, CONTRACT_NAME, CONTRACT_VERSION};

    use cosmwasm_std::entry_point;
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
    use cw2::set_contract_version;

    // This makes a conscious choice on the various generics used by the contract
    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        Cw721GovernedCollectionProxy::default().instantiate(deps, env, info, msg)
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        Cw721GovernedCollectionProxy::default().execute(deps, env, info, msg)
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        Cw721GovernedCollectionProxy::default().query(deps, env, msg)
    }

    #[entry_point]
    pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> StdResult<Response> {
        // Set contract to version to latest
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        match msg {
            MigrateMsg::WithUpdate {
                whitelist,
                transfer_fee,
                origin,
            } => {
                if let Some(list) = whitelist.clone() {
                    list.iter()
                        .map(|item| {
                            Cw721GovernedCollectionProxy::default()
                                .whitelist
                                .add(deps.storage, &item.to_string())
                        })
                        .collect::<StdResult<Vec<_>>>()?;
                }
                let res = Cw721GovernedCollectionProxy::default().governance.migrate(
                    deps,
                    env,
                    cw721_governed_proxy::msg::MigrateMsg::WithUpdate {
                        origin,
                        transfer_fee,
                    },
                )?;
                Ok(res.add_attribute("whitelist", format!("{:?}", whitelist)))
            }
        }
    }
}
