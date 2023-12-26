#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg,
};
use cw2::set_contract_version;
use cw721_outgoing_proxy::ProxyExecuteMsg;

use cw_rate_limiter::{Rate, RateLimitError};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{ORIGIN, RATE_LIMIT};

const CONTRACT_NAME: &str = "crates.io:cw721-outgoing-proxy-rate-limit";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, RateLimitError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    ORIGIN.save(
        deps.storage,
        &msg.origin
            .map(|a| deps.api.addr_validate(&a))
            .transpose()?
            .unwrap_or(info.sender),
    )?;
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, RateLimitError> {
    match msg {
        ExecuteMsg::ReceiveNft(msg) => execute_receive_nft(deps, env, info, msg),
    }
}

pub fn execute_receive_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: cw721::Cw721ReceiveMsg,
) -> Result<Response, RateLimitError> {
    RATE_LIMIT.limit(deps.storage, &env.block, info.sender.as_str())?;
    Ok(Response::default().add_message(WasmMsg::Execute {
        contract_addr: ORIGIN.load(deps.storage)?.into_string(),
        msg: to_json_binary(&ProxyExecuteMsg::ReceiveProxyNft {
            eyeball: info.sender.into_string(),
            msg,
        })?,
        funds: vec![],
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::RateLimit {} => to_json_binary(&RATE_LIMIT.query_limit(deps.storage)?),
        QueryMsg::Origin {} => to_json_binary(&ORIGIN.load(deps.storage)?),
    }
}
