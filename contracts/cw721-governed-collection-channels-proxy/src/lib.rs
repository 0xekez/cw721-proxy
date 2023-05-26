use cosmwasm_std::{from_binary, Binary, DepsMut, Env, MessageInfo, Response, Storage};
use cw721::Cw721ReceiveMsg;
use error::ContractError;
use ibc_outgoing_msg::IbcOutgoingMsg;
use state::WHITELIST;

pub mod error;
pub mod msg;
pub mod state;

#[cfg(test)]
mod tests;

#[cfg(not(feature = "library"))]
pub mod entry {
    use crate::error::ContractError;
    use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
    use crate::state::{CONTRACT_NAME, CONTRACT_VERSION, WHITELIST};
    use crate::{
        execute_add_to_whitelist, execute_bridge_nft, execute_clear_whitelist, execute_receive_nft,
        execute_remove_from_whitelist,
    };

    use cosmwasm_std::{entry_point, to_binary, Order};
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
    use cw2::set_contract_version;
    use cw_ics721_governance::Action;

    // This makes a conscious choice on the various generics used by the contract
    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        if let Some(list) = msg.whitelist.clone() {
            list.iter()
                .map(|item| {
                    deps.api.addr_validate(item.0.as_str())?;
                    WHITELIST.save(deps.storage, item.0.to_string(), &item.1)
                })
                .collect::<StdResult<Vec<_>>>()?;
        }
        let res =
            cw_ics721_governance::instantiate(deps, info, msg.owner, msg.origin, msg.transfer_fee)?;
        Ok(res.add_attribute(
            "whitelist".to_string(),
            msg.whitelist
                .map_or("none".to_string(), |w| format!("{:?}", w)),
        ))
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        match msg {
            ExecuteMsg::Governance(Action::BridgeNft {
                collection,
                token_id,
                msg,
            }) => execute_bridge_nft(deps, env, info, collection, token_id, msg),
            ExecuteMsg::Governance(action) => {
                Ok(cw_ics721_governance::execute(deps, env, &info, action)?)
            }
            ExecuteMsg::ReceiveNft(msg) => execute_receive_nft(deps, env, info, msg),
            ExecuteMsg::AddToWhitelist {
                collection,
                channels,
            } => execute_add_to_whitelist(deps, env, info, collection, channels),
            ExecuteMsg::RemoveFromWhitelist { collection } => {
                execute_remove_from_whitelist(deps, env, info, collection)
            }
            ExecuteMsg::ClearWhitelist() => execute_clear_whitelist(deps, env, info),
        }
    }

    #[entry_point]
    pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::Governance() => cw_ics721_governance::query_governance(deps.storage),
            QueryMsg::Whitelist {} => to_binary(
                &WHITELIST
                    .map
                    .range(deps.storage, None, None, Order::Ascending)
                    .collect::<StdResult<Vec<(String, Vec<String>)>>>()?,
            ),
            QueryMsg::Whitelisted {
                collection,
                channel,
            } => to_binary(&WHITELIST.query_is_whitelisted(
                deps.storage,
                collection,
                |channels| channels.contains(&channel),
            )?),
        }
    }

    #[entry_point]
    pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> StdResult<Response> {
        // Set contract to version to latest
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        match msg {
            MigrateMsg::WithUpdate {
                origin,
                owner,
                transfer_fee,
                whitelist,
            } => {
                if let Some(list) = whitelist.clone() {
                    list.iter()
                        .map(|item| WHITELIST.save(deps.storage, item.0.clone(), &item.1))
                        .collect::<StdResult<Vec<_>>>()?;
                }
                let res = cw_ics721_governance::migrate(deps, owner, origin, transfer_fee)?;
                Ok(res.add_attribute(
                    "whitelist".to_string(),
                    whitelist.map_or("none".to_string(), |w| format!("{:?}", w)),
                ))
            }
        }
    }
}

pub fn execute_add_to_whitelist(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection: String,
    channels: Vec<String>,
) -> Result<Response, ContractError> {
    cw_ics721_governance::assert_owner(deps.storage, &info.sender)?;
    WHITELIST.save(deps.storage, collection.clone(), &channels)?;
    Ok(Response::default()
        .add_attribute("method", "execute_add_to_whitelist")
        .add_attribute("collection", collection)
        .add_attribute("channels", format!("{:?}", channels)))
}

pub fn execute_remove_from_whitelist(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection: String,
) -> Result<Response, ContractError> {
    cw_ics721_governance::assert_owner(deps.storage, &info.sender)?;
    WHITELIST.remove(deps.storage, collection.clone());
    Ok(Response::default()
        .add_attribute("method", "execute_remove_from_whitelist")
        .add_attribute("key", collection))
}

pub fn execute_clear_whitelist(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    cw_ics721_governance::assert_owner(deps.storage, &info.sender)?;
    WHITELIST.clear(deps.storage)?;
    Ok(Response::default().add_attribute("method", "execute_clear_whitelist"))
}

pub fn execute_receive_nft(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let IbcOutgoingMsg {
        channel_id,
        memo: _,
        receiver: _,
        timeout: _,
    } = from_binary(&msg.msg)?;
    is_whitelisted(deps.storage, info.sender.to_string(), channel_id)?;
    Ok(cw_ics721_governance::execute_receive_nft(deps, info, msg)?)
}

pub fn execute_bridge_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection: String,
    token_id: String,
    msg: Binary,
) -> Result<Response, ContractError> {
    let IbcOutgoingMsg {
        channel_id,
        memo: _,
        receiver: _,
        timeout: _,
    } = from_binary(&msg)?;
    is_whitelisted(deps.storage, collection.clone(), channel_id)?;
    Ok(cw_ics721_governance::execute_bridge_nft(
        deps, env, &info, collection, token_id, msg,
    )?)
}

pub fn is_whitelisted(
    storage: &dyn Storage,
    collection: String,
    channel: String,
) -> Result<(), ContractError> {
    if !WHITELIST.has(storage, collection.clone()) {
        Err(ContractError::NotWhitelisted {
            requestee: collection,
        })
    } else {
        match WHITELIST
            .query_is_whitelisted(storage, collection, |channels| channels.contains(&channel))?
        {
            true => Ok(()),
            false => Err(ContractError::NotWhitelisted { requestee: channel }),
        }
    }
}
