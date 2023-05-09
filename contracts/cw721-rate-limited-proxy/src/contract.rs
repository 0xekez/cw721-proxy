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
use crate::state::{ORIGIN, RATE_LIMIT};

const CONTRACT_NAME: &str = "crates.io:cw721-proxy-rate-limit";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    ORIGIN.save(
        deps.storage,
        &msg.origin
            .map(|a| deps.api.addr_validate(&a))
            .transpose()?
            .unwrap_or(info.sender),
    )?;
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
            .add_attribute("units", units))
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
    }
}

pub fn execute_receive_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: cw721::Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    RATE_LIMIT.limit(deps.storage, &env, info.sender.as_str())?;
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
        QueryMsg::RateLimit {} => to_binary(&RATE_LIMIT.query_limit(deps.storage)?),
        QueryMsg::Origin {} => to_binary(&ORIGIN.load(deps.storage)?),
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
                ORIGIN.save(deps.storage, &origin)?;
            }
            Ok(Response::default().add_attribute("method", "migrate"))
        }
    }
}
