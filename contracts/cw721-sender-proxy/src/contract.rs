use cosmwasm_std::Addr;
use cosmwasm_std::Binary;
use cosmwasm_std::Deps;
use cosmwasm_std::StdResult;
use cosmwasm_std::Storage;
use cosmwasm_std::WasmMsg;
use cosmwasm_std::to_binary;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, MessageInfo};
use cosmwasm_std::entry_point;
use cosmwasm_std::Response;
use cw2::set_contract_version;
use cw721_proxy::ProxyExecuteMsg;

use crate::msg::ExecuteMsg;
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
    let origin = msg.origin
        .map(|a| deps.api.addr_validate(&a))
        .transpose()?
        .unwrap_or(info.sender);
    ORIGIN.save(
        deps.storage,
        &origin,
    )?;
    WHITELIST.init(deps, msg.whitelist)?;
    Ok(Response::default()
        .add_attribute("method", "instantiate")
        .add_attribute("origin", origin)
    )
}

pub fn is_owner(storage: &dyn Storage, addr: &Addr) -> Result<(), ContractError> {
    if addr != &OWNER.load(storage).unwrap() {
        return Err(ContractError::Unauthorized { addr: addr.to_string() })
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
        ExecuteMsg::AddToWhitelist(addr) => execute_add_to_whitelist(deps, env, info, &addr),
        ExecuteMsg::RemoveFromWhitelist(addr) => execute_remove_from_whitelist(deps, env, info, &addr),
    }
}

pub fn execute_add_to_whitelist(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    addr: &str,
) -> Result<Response, ContractError> {
    is_owner(deps.storage, &info.sender)?;
    let addr = deps.api.addr_validate(addr)?;
    WHITELIST.add(deps.storage, &addr.to_string())?;
    Ok(Response::default()
        .add_attribute("method", "execute_add_to_whitelist")
        .add_attribute("addr", addr)
    )
}

pub fn execute_remove_from_whitelist(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    addr: &str,
) -> Result<Response, ContractError> {
    is_owner(deps.storage, &info.sender)?;
    let addr = deps.api.addr_validate(addr)?;
    WHITELIST.remove(deps.storage, &addr.to_string())?;
    Ok(Response::default()
        .add_attribute("method", "execute_remove_from_whitelist")
        .add_attribute("addr", addr)
    )
}

pub fn execute_receive_nft(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: cw721::Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    is_whitelisted(deps.storage, &info.sender.to_string())?;
    Ok(Response::default().add_message(WasmMsg::Execute {
        contract_addr: ORIGIN.load(deps.storage)?.into_string(),
        msg: to_binary(&ProxyExecuteMsg::ReceiveProxyNft {
            eyeball: info.sender.into_string(),
            msg,
        })?,
        funds: vec![],
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Origin {} => to_binary(&ORIGIN.load(deps.storage)?),
        QueryMsg::Whitelist {} => to_binary(&WHITELIST.query_whitelist(deps.storage)?),
        QueryMsg::WhiteListed(addr) => to_binary(&WHITELIST.query_is_whitelisted(deps.storage, &addr)?),
    }
}

pub fn is_whitelisted(storage: &dyn Storage, addr: &String) -> Result<(), ContractError> {
    match WHITELIST.query_is_whitelisted(storage, addr)? {
        true => Ok(()),
        false => Err(ContractError::Unauthorized { addr: addr.to_string() }),
    }
}