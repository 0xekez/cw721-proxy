use cosmwasm_schema::{cw_serde, QueryResponses};

use cw_rate_limiter::Rate;

use cosmwasm_std::Coin;
use cw_ics721_governance::{cw_ics721_governance_execute, cw_ics721_governance_query};

#[cw_serde]
pub struct InstantiateMsg {
    pub origin: Option<String>,
    pub owner: Option<String>,
    pub transfer_fee: Option<Coin>,
    pub rate_limit: Rate,
}

#[cw_ics721_governance_execute]
#[cw_serde]
pub enum ExecuteMsg {
    RateLimit(Rate),
}

#[cw_ics721_governance_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Gets the contract's rate limit.
    #[returns(Rate)]
    RateLimit {},
}

#[cw_serde]
pub enum MigrateMsg {
    WithUpdate {
        origin: Option<String>,
        owner: Option<String>,
        transfer_fee: Option<Coin>,
        rate_limit: Option<Rate>,
    },
}
