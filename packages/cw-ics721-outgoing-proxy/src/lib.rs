use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    from_json, to_json_binary, DepsMut, Empty, Env, MessageInfo, Reply, Response, StdError,
    StdResult, SubMsg, WasmMsg,
};
use cw721::Cw721ReceiveMsg;
use cw721_base::ExecuteMsg;
use cw_ics721_outgoing_proxy_derive::cw721_receive_nft;
use ics721_types::ibc_types::{IbcOutgoingMsg, IbcOutgoingProxyMsg};

pub(crate) const DO_NOTHING_REPLY_ID: u64 = 1;

#[cw721_receive_nft]
#[cw_serde]
pub enum ProxyExecuteMsg {}

/// NFT send to this proxy contract, so we need to:
/// - transfer NFT to the ics721 contract (origin)
/// - call receive_nft on the ics721 contract
pub fn execute_receive_nft(
    info: MessageInfo,
    msg: Cw721ReceiveMsg,
    ics721: String,
) -> StdResult<Response> {
    let Cw721ReceiveMsg {
        sender: nft_owner,
        token_id,
        msg,
    } = msg;
    let IbcOutgoingMsg {
        channel_id,
        memo,
        receiver,
        ..
    } = from_json(&msg)?;

    let msg = IbcOutgoingProxyMsg {
        collection: info.sender.to_string(),
        msg,
    };
    let transfer_msg = WasmMsg::Execute {
        contract_addr: info.sender.to_string(), // collection contract
        msg: to_json_binary(&ExecuteMsg::<Empty, Empty>::TransferNft {
            recipient: ics721.to_string(), // ics721
            token_id: token_id.clone(),
        })?,
        funds: vec![],
    };
    let transfer_msg = SubMsg::reply_on_success(transfer_msg, DO_NOTHING_REPLY_ID);
    let ics721_msg = WasmMsg::Execute {
        contract_addr: ics721.to_string(), // ics721
        msg: to_json_binary(&ProxyExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
            sender: nft_owner.clone(),
            token_id: token_id.clone(),
            msg: to_json_binary(&msg)?,
        }))?,
        funds: vec![],
    };
    let ics721_msg = SubMsg::reply_on_success(ics721_msg, DO_NOTHING_REPLY_ID);
    Ok(Response::default()
        .add_submessage(transfer_msg)
        .add_submessage(ics721_msg)
        .add_attribute("nft_owner", nft_owner)
        .add_attribute("collection", info.sender.to_string())
        .add_attribute("token_id", token_id)
        .add_attribute("channel_id", channel_id)
        .add_attribute("memo", memo.unwrap_or("empty".to_string()))
        .add_attribute("receiver", receiver))
}

pub fn reply(_deps: DepsMut, _env: Env, reply: Reply) -> StdResult<Response> {
    match reply.id {
        DO_NOTHING_REPLY_ID => Ok(Response::default()),
        _ => Err(StdError::generic_err("reply id not recognized")),
    }
}
