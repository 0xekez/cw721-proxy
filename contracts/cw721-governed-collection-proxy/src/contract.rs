#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::to_binary;
use cosmwasm_std::Binary;
use cosmwasm_std::Deps;
use cosmwasm_std::Response;
use cosmwasm_std::StdResult;
use cosmwasm_std::Storage;
use cosmwasm_std::WasmMsg;
use cosmwasm_std::{DepsMut, Env, MessageInfo};
use cw2::set_contract_version;
use cw721_proxy::ProxyExecuteMsg;

use crate::msg::ExecuteMsg;
use crate::msg::MigrateMsg;
use crate::msg::QueryMsg;
use crate::state::GOVERNANCE;
use crate::{error::ContractError, msg::InstantiateMsg, state::WHITELIST};
use cw721_governed_proxy::ContractError as GovernanceContractError;

pub const CONTRACT_NAME: &str = "crates.io:cw721-proxy-sender";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    GOVERNANCE.save_owner(deps.storage, &Some(info.sender.clone()))?;

    let origin = msg
        .origin
        .map(|a| deps.api.addr_validate(&a))
        .transpose()?
        .unwrap_or(info.sender);
    GOVERNANCE.save_origin(deps.storage, &origin)?;

    WHITELIST.init(deps, msg.whitelist.clone())?;

    Ok(Response::default()
        .add_attribute("method", "instantiate")
        .add_attribute("origin", origin)
        .add_attribute(
            "whitelist",
            format!("{:?}", msg.whitelist.unwrap_or_default()),
        ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ReceiveNft(msg) => execute_receive_nft(deps, env, info, msg),
        ExecuteMsg::AddToWhitelist { sender } => execute_add_to_whitelist(deps, env, info, &sender),
        ExecuteMsg::RemoveFromWhitelist { sender } => {
            execute_remove_from_whitelist(deps, env, info, &sender)
        }
        ExecuteMsg::Origin(origin) => execute_origin(deps, env, info, origin),
        ExecuteMsg::Owner(owner) => execute_owner(deps, env, info, owner),
    }
}

pub fn execute_owner(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    addr: String,
) -> Result<Response, ContractError> {
    GOVERNANCE.is_owner(deps.storage, info.sender)?;
    let owner = deps.api.addr_validate(&addr)?;
    GOVERNANCE.save_owner(deps.storage, &Some(owner))?;
    Ok(Response::default()
        .add_attribute("method", "execute_owner")
        .add_attribute("owner", addr))
}

pub fn execute_add_to_whitelist(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    sender: &str,
) -> Result<Response, ContractError> {
    GOVERNANCE.is_owner(deps.storage, info.sender)?;
    let sender = deps.api.addr_validate(sender)?;
    WHITELIST.add(deps.storage, &sender.to_string())?;
    Ok(Response::default()
        .add_attribute("method", "execute_add_to_whitelist")
        .add_attribute("sender", sender))
}

pub fn execute_remove_from_whitelist(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    sender: &str,
) -> Result<Response, ContractError> {
    GOVERNANCE.is_owner(deps.storage, info.sender)?;
    let sender = deps.api.addr_validate(sender)?;
    WHITELIST.remove(deps.storage, &sender.to_string())?;
    Ok(Response::default()
        .add_attribute("method", "execute_remove_from_whitelist")
        .add_attribute("sender", sender))
}

pub fn execute_receive_nft(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: cw721::Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    is_whitelisted(deps.storage, &info.sender.to_string())?;
    // transfer NFT to ICS721
    let cw721::Cw721ReceiveMsg {
        token_id,
        sender: _,
        msg: _,
    } = msg.clone();
    let transfer_nft_msg = WasmMsg::Execute {
        contract_addr: info.sender.to_string(), // sender is collection
        msg: to_binary(&cw721::Cw721ExecuteMsg::TransferNft {
            recipient: GOVERNANCE.load_origin(deps.storage)?.into_string(),
            token_id,
        })?,
        funds: vec![],
    };
    // forward Cw721ReceiveMsg to ICS721
    let receive_msg = WasmMsg::Execute {
        contract_addr: GOVERNANCE.load_origin(deps.storage)?.into_string(),
        msg: to_binary(&ProxyExecuteMsg::ReceiveProxyNft {
            eyeball: info.sender.into_string(),
            msg,
        })?,
        funds: vec![],
    };
    Ok(Response::default().add_messages(vec![transfer_nft_msg, receive_msg]))
}

pub fn execute_origin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    addr: String,
) -> Result<Response, ContractError> {
    let addr = deps.api.addr_validate(&addr)?;
    GOVERNANCE.is_owner(deps.storage, info.sender)?;
    GOVERNANCE.save_origin(deps.storage, &addr)?;
    Ok(Response::default()
        .add_attribute("method", "execute_origin")
        .add_attribute("origin", addr))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Owner {} => to_binary(&GOVERNANCE.load_owner(deps.storage)?),
        QueryMsg::Origin {} => to_binary(&GOVERNANCE.load_origin(deps.storage)?),
        QueryMsg::Whitelist {} => to_binary(&WHITELIST.query_whitelist(deps.storage)?),
        QueryMsg::WhiteListed { sender } => {
            to_binary(&WHITELIST.query_is_whitelisted(deps.storage, &sender)?)
        }
    }
}

pub fn is_whitelisted(storage: &dyn Storage, addr: &String) -> Result<(), ContractError> {
    match WHITELIST.query_is_whitelisted(storage, addr)? {
        true => Ok(()),
        false => Err(ContractError::GovernanceError(
            GovernanceContractError::Unauthorized {
                addr: addr.to_string(),
            },
        )),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    // Set contract to version to latest
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    match msg {
        MigrateMsg::WithUpdate { origin, whitelist } => {
            if let Some(list) = whitelist {
                list.iter()
                    .map(|item| {
                        deps.api.addr_validate(item.as_str())?;
                        WHITELIST.add(deps.storage, &item.to_string())
                    })
                    .collect::<StdResult<Vec<_>>>()?;
            }
            if let Some(origin) = origin {
                let origin = deps.api.addr_validate(&origin)?;
                GOVERNANCE.save_origin(deps.storage, &origin)?;
            }
            Ok(Response::default().add_attribute("method", "migrate"))
        }
    }
}
