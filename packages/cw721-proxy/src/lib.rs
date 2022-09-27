use cosmwasm_schema::cw_serde;
use cw721_proxy_derive::cw721_proxy;

#[cw721_proxy]
#[cw_serde]
pub enum ProxyExecuteMsg {}
