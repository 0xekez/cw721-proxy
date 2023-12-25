use cosmwasm_schema::cw_serde;
use cw721_outgoing_proxy_derive::cw721_outgoing_proxy;

#[cw721_outgoing_proxy]
#[cw_serde]
pub enum ProxyExecuteMsg {}
