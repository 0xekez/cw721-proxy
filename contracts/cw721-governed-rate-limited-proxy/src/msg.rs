use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::Addr;
use cw_rate_limiter::Rate;

use cosmwasm_std::{Binary, Coin};

#[cw_serde]
pub struct InstantiateMsg {
    pub origin: Option<String>,
    pub transfer_fee: Option<Coin>,
    pub rate_limit: Rate,
}

#[cw_serde]
pub enum ExecuteMsg {
    // -- copied from cw721-governed-proxy
    Owner(String),
    Origin(String),

    /// Transfer fee for bridging nft
    TransferFee(Option<Coin>),

    /// Send funds from proxy to address
    SendFunds {
        to_address: String,
        amount: Coin,
    },

    ReceiveNft(cw721::Cw721ReceiveMsg),
    /// analogous to SendNft from cw721_base
    BridgeNft {
        collection: String,
        token_id: String,
        msg: Binary,
    },
    // ----

    // -- whitelist specifics
    RateLimit(Rate),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // -- copied from governance proxy
    #[returns(Addr)]
    Owner {},

    #[returns(Addr)]
    Origin {},

    #[returns(Option<Coin>)]
    TransferFee {},
    // ----
    /// Gets the contract's rate limit.
    #[returns(Rate)]
    RateLimit {},
}

#[cw_serde]
pub enum MigrateMsg {
    WithUpdate {
        rate: Option<Rate>,
        transfer_fee: Option<Coin>,
        origin: Option<String>,
    },
}