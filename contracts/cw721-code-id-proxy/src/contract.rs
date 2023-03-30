use cosmwasm_std::entry_point;
use cosmwasm_std::to_binary;
use cosmwasm_std::Addr;
use cosmwasm_std::Binary;
use cosmwasm_std::Deps;
use cosmwasm_std::Response;
use cosmwasm_std::StdResult;
use cosmwasm_std::Storage;
use cosmwasm_std::WasmMsg;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, MessageInfo};
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
        ExecuteMsg::AddToWhitelist { code_id } => {
            execute_add_to_whitelist(deps, env, info, &code_id)
        }
        ExecuteMsg::RemoveFromWhitelist { code_id } => {
            execute_remove_from_whitelist(deps, env, info, &code_id)
        }
    }
}

pub fn execute_add_to_whitelist(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    code_id: &u64,
) -> Result<Response, ContractError> {
    is_owner(deps.storage, &info.sender)?;
    WHITELIST.add(deps.storage, code_id)?;
    Ok(Response::default()
        .add_attribute("method", "execute_add_to_whitelist")
        .add_attribute("code_id", code_id.to_string()))
}

pub fn execute_remove_from_whitelist(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    code_id: &u64,
) -> Result<Response, ContractError> {
    is_owner(deps.storage, &info.sender)?;
    WHITELIST.remove(deps.storage, code_id)?;
    Ok(Response::default()
        .add_attribute("method", "execute_remove_from_whitelist")
        .add_attribute("code_id", code_id.to_string()))
}

pub fn execute_receive_nft(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: cw721::Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let contract_info = deps.querier.query_wasm_contract_info(info.sender.clone())?;
    is_whitelisted(deps.storage, &contract_info.code_id)?;
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
        QueryMsg::WhiteListed { code_id } => {
            to_binary(&WHITELIST.query_is_whitelisted(deps.storage, &code_id)?)
        }
    }
}

pub fn is_whitelisted(storage: &dyn Storage, code_id: &u64) -> Result<(), ContractError> {
    match WHITELIST.query_is_whitelisted(storage, code_id)? {
        true => Ok(()),
        false => Err(ContractError::UnauthorizedCodeId { code_id: *code_id }),
    }
}
