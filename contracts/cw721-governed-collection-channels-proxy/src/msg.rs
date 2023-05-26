use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;
use cw_ics721_governance::{cw_ics721_governance_execute, cw_ics721_governance_query};

#[cw_serde]
pub struct InstantiateMsg {
    pub origin: Option<String>,
    pub owner: Option<String>,
    pub transfer_fee: Option<Coin>,
    pub whitelist: Option<Vec<(String, Vec<String>)>>,
}

#[cw_ics721_governance_execute]
#[cw_serde]
pub enum ExecuteMsg {
    AddToWhitelist {
        collection: String,
        channels: Vec<String>,
    },
    RemoveFromWhitelist {
        collection: String,
    },
    ClearWhitelist(),
}

#[cw_ics721_governance_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Gets a list of collection and channels authorized for ICS721 transfers.
    #[returns(Vec<(String, Vec<String>)>)]
    Whitelist {},

    /// True in case CW721 contract and channel is authorized for ICS721 transfers.
    #[returns(bool)]
    Whitelisted { collection: String, channel: String },
}

#[cw_serde]
pub enum MigrateMsg {
    WithUpdate {
        origin: Option<String>,
        owner: Option<String>,
        transfer_fee: Option<Coin>,
        whitelist: Option<Vec<(String, Vec<String>)>>,
    },
}
