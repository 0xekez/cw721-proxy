use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;
use cw_ics721_governance::{cw_ics721_governance_execute, cw_ics721_governance_query};

#[cw_serde]
pub struct InstantiateMsg {
    pub origin: Option<String>,
    pub owner: Option<String>,
    pub transfer_fee: Option<Coin>,
    pub whitelist: Option<Vec<u64>>,
}

#[cw_ics721_governance_execute]
#[cw_serde]
pub enum ExecuteMsg {
    AddToWhitelist { value: u64 },
    RemoveFromWhitelist { value: u64 },
    ClearWhitelist(),
}

#[cw_ics721_governance_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<u64>)]
    Whitelist {},

    #[returns(bool)]
    Whitelisted { value: u64 },
}

#[cw_serde]
pub enum MigrateMsg {
    WithUpdate {
        origin: Option<String>,
        owner: Option<String>,
        transfer_fee: Option<Coin>,
        whitelist: Option<Vec<u64>>,
    },
}
