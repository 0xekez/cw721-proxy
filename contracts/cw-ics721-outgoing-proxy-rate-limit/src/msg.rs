use cosmwasm_schema::{cw_serde, QueryResponses};

use cw_ics721_outgoing_proxy_derive::cw721_receive_nft;
use cw_rate_limiter::Rate;

#[cw_serde]
pub struct InstantiateMsg {
    pub rate_limit: Rate,
    pub origin: Option<String>,
}

#[cw721_receive_nft]
#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Gets the contract's rate limit.
    #[returns(Rate)]
    RateLimit {},

    #[returns(String)]
    Origin {},
}
