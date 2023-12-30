use cosmwasm_schema::cw_serde;
use cw721_outgoing_proxy_derive::cw721_receive_nft;

#[cw721_receive_nft]
#[cw_serde]
pub enum ProxyExecuteMsg {}
