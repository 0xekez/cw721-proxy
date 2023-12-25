use cosmwasm_schema::{cw_serde, QueryResponses};

use cw_incoming_proxy_derive::{cw_incoming_proxy_execute, cw_incoming_proxy_query};

#[cw_serde]
pub struct InstantiateMsg {
    pub origin: Option<String>,
    pub source_channels: Option<Vec<String>>,
}

#[cw_incoming_proxy_execute]
#[cw_serde]
pub enum ExecuteMsg {}

#[cw_incoming_proxy_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
