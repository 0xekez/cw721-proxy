#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::to_binary;
use cosmwasm_std::Addr;
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
use crate::state::ORIGIN;
use crate::state::OWNER;
use crate::{error::ContractError, msg::InstantiateMsg, state::WHITELIST};

const CONTRACT_NAME: &str = "crates.io:cw721-proxy-sender-whitelist";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    OWNER.save(deps.storage, &info.sender)?;
    let origin = msg
        .origin
        .map(|a| deps.api.addr_validate(&a))
        .transpose()?
        .unwrap_or(info.sender);
    ORIGIN.save(deps.storage, &origin)?;
    WHITELIST.init(deps, msg.whitelist)?;
    Ok(Response::default()
        .add_attribute("method", "instantiate")
        .add_attribute("origin", origin))
}

pub fn is_owner(storage: &dyn Storage, addr: &Addr) -> Result<(), ContractError> {
    if addr != &OWNER.load(storage).unwrap() {
        return Err(ContractError::Unauthorized {
            addr: addr.to_string(),
        });
    }
    Ok(())
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
    }
}

pub fn execute_add_to_whitelist(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    sender: &str,
) -> Result<Response, ContractError> {
    is_owner(deps.storage, &info.sender)?;
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
    is_owner(deps.storage, &info.sender)?;
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
            recipient: ORIGIN.load(deps.storage)?.into_string(),
            token_id,
        })?,
        funds: vec![],
    };
    // forward Cw721ReceiveMsg to ICS721
    let receive_msg = WasmMsg::Execute {
        contract_addr: ORIGIN.load(deps.storage)?.into_string(),
        msg: to_binary(&ProxyExecuteMsg::ReceiveProxyNft {
            eyeball: info.sender.into_string(),
            msg,
        })?,
        funds: vec![],
    };
    Ok(Response::default().add_messages(vec![transfer_nft_msg, receive_msg]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Origin {} => to_binary(&ORIGIN.load(deps.storage)?),
        QueryMsg::Whitelist {} => to_binary(&WHITELIST.query_whitelist(deps.storage)?),
        QueryMsg::WhiteListed { sender } => {
            to_binary(&WHITELIST.query_is_whitelisted(deps.storage, &sender)?)
        }
    }
}

pub fn is_whitelisted(storage: &dyn Storage, addr: &String) -> Result<(), ContractError> {
    match WHITELIST.query_is_whitelisted(storage, addr)? {
        true => Ok(()),
        false => Err(ContractError::Unauthorized {
            addr: addr.to_string(),
        }),
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
                ORIGIN.save(deps.storage, &origin)?;
            }
            Ok(Response::default().add_attribute("method", "migrate"))
        }
    }
}
