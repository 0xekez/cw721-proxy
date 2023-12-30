#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Reply, Response, StdError,
    StdResult, SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use cw721::Cw721ReceiveMsg;
use cw721_base::msg::ExecuteMsg as Cw721ExecuteMsg;
use ics721_types::ibc_types::IbcOutgoingProxyMsg;

use cw_rate_limiter::{Rate, RateLimitError};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{ORIGIN, RATE_LIMIT};

const CONTRACT_NAME: &str = "crates.io:cw721-outgoing-proxy-rate-limit";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) const DO_NOTHING_REPLY_ID: u64 = 1;

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
    msg: Cw721ReceiveMsg,
) -> Result<Response, RateLimitError> {
    RATE_LIMIT.limit(deps.storage, &env.block, info.sender.as_str())?;
    // NFT send to this proxy contract, so we need to:
    // - transfer NFT to the origin contract
    // - call receive_nft on the origin contract
    let Cw721ReceiveMsg {
        sender,
        token_id,
        msg,
    } = msg;
    let msg = IbcOutgoingProxyMsg {
        collection: info.sender.to_string(),
        msg,
    };
    let transfer_msg = WasmMsg::Execute {
        contract_addr: info.sender.to_string(), // collection contract
        msg: to_json_binary(&Cw721ExecuteMsg::<Empty, Empty>::TransferNft {
            recipient: ORIGIN.load(deps.storage)?.into_string(), // ics721
            token_id: token_id.clone(),
        })?,
        funds: vec![],
    };
    let transfer_msg = SubMsg::reply_on_success(transfer_msg, DO_NOTHING_REPLY_ID);
    let ics721_msg = WasmMsg::Execute {
        contract_addr: ORIGIN.load(deps.storage)?.into_string(), // ics721
        msg: to_json_binary(&ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
            sender,
            token_id,
            msg: to_json_binary(&msg)?,
        }))?,
        funds: vec![],
    };
    let ics721_msg = SubMsg::reply_on_success(ics721_msg, DO_NOTHING_REPLY_ID);
    Ok(Response::default()
        .add_submessage(transfer_msg)
        .add_submessage(ics721_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, reply: Reply) -> StdResult<Response> {
    match reply.id {
        DO_NOTHING_REPLY_ID => Ok(Response::default()),
        _ => Err(StdError::generic_err("reply id not recognized")),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::RateLimit {} => to_json_binary(&RATE_LIMIT.query_limit(deps.storage)?),
        QueryMsg::Origin {} => to_json_binary(&ORIGIN.load(deps.storage)?),
    }
}
