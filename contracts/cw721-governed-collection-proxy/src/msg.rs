use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;
use cw_ics721_governance::{cw_ics721_governance_execute, cw_ics721_governance_query};

#[cw_serde]
pub struct InstantiateMsg {
    pub origin: Option<String>,
    pub owner: Option<String>,
    pub transfer_fee: Option<Coin>,
    pub whitelist: Option<Vec<String>>,
}

#[cw_ics721_governance_execute]
#[cw_serde]
pub enum ExecuteMsg {
    AddToWhitelist { value: String },
    RemoveFromWhitelist { value: String },
    ClearWhitelist(),
}

#[cw_ics721_governance_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<String>)]
    Whitelist {},

    #[returns(bool)]
    Whitelisted { value: String },
}

#[cw_serde]
pub enum MigrateMsg {
    WithUpdate {
        origin: Option<String>,
        owner: Option<String>,
        transfer_fee: Option<Coin>,
        whitelist: Option<Vec<String>>,
    },
}
