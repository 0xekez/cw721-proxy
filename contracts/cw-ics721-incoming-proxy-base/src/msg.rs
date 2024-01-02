use cosmwasm_schema::{cw_serde, QueryResponses};

use cw_ics721_incoming_proxy_derive::{ics721_incoming_proxy_execute, ics721_incoming_proxy_query};

#[cw_serde]
pub struct InstantiateMsg {
    pub origin: Option<String>,
    pub channels: Option<Vec<String>>,
}

#[ics721_incoming_proxy_execute]
#[cw_serde]
pub enum ExecuteMsg {}

#[ics721_incoming_proxy_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}

#[cw_serde]
pub enum MigrateMsg {
    WithUpdate {
        origin: Option<String>,
        channels: Option<Vec<String>>,
    },
}
