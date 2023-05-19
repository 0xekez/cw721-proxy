#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg,
};
use cw2::set_contract_version;
use cw721_proxy::ProxyExecuteMsg;

use cw_rate_limiter::Rate;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::GOVERNANCE;
use crate::state::RATE_LIMIT;

pub const CONTRACT_NAME: &str = "crates.io:cw721-proxy-rate-limit";
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

    if msg.rate_limit.is_zero() {
        Err(ContractError::ZeroRate {})
    } else {
        let (rate, units) = match msg.rate_limit {
            Rate::PerBlock(rate) => (rate, "nfts_per_block"),
            Rate::Blocks(rate) => (rate, "blocks_per_nft"),
        };
        RATE_LIMIT.init(deps.storage, &msg.rate_limit)?;
        Ok(Response::default()
            .add_attribute("method", "instantiate")
            .add_attribute("rate", rate.to_string())
            .add_attribute("units", units)
            .add_attribute("origin", origin))
    }
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
        ExecuteMsg::RateLimit(rate_limit) => execute_rate_limit(deps, env, info, rate_limit),
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

pub fn execute_receive_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: cw721::Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    RATE_LIMIT.limit(deps.storage, &env, info.sender.as_str())?;
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

pub fn execute_rate_limit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    rate_limit: Rate,
) -> Result<Response, ContractError> {
    GOVERNANCE.is_owner(deps.storage, info.sender)?;
    if rate_limit.is_zero() {
        Err(ContractError::ZeroRate {})
    } else {
        RATE_LIMIT.init(deps.storage, &rate_limit)?;
        let (rate, units) = match rate_limit {
            Rate::PerBlock(rate) => (rate, "nfts_per_block"),
            Rate::Blocks(rate) => (rate, "blocks_per_nft"),
        };
        Ok(Response::default()
            .add_attribute("method", "execute_rate_limit")
            .add_attribute("rate", rate.to_string())
            .add_attribute("units", units))
    }
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
        QueryMsg::RateLimit {} => to_binary(&RATE_LIMIT.query_limit(deps.storage)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    // Set contract to latest version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    match msg {
        MigrateMsg::WithUpdate { origin, rate_limit } => {
            if let Some(rate) = rate_limit {
                if rate.is_zero() {
                    return Err(ContractError::ZeroRate {});
                } else {
                    RATE_LIMIT.init(deps.storage, &rate)?;
                }
            }
            if let Some(origin) = origin {
                let origin = deps.api.addr_validate(&origin)?;
                GOVERNANCE.save_origin(deps.storage, &origin)?;
            }
            Ok(Response::default().add_attribute("method", "migrate"))
        }
    }
}
