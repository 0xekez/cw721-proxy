use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::Addr;
use cw_rate_limiter::Rate;

#[cw_serde]
pub struct InstantiateMsg {
    pub rate_limit: Rate,
    pub origin: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    ReceiveNft(cw721::Cw721ReceiveMsg),
    RateLimit(Rate),
    Origin(Addr),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Addr)]
    Owner {},

    /// Gets the contract's rate limit.
    #[returns(Rate)]
    RateLimit {},

    #[returns(String)]
    Origin {},
}

#[cw_serde]
pub enum MigrateMsg {
    WithUpdate {
        rate_limit: Option<Rate>,
        origin: Option<String>,
    },
}
