use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Coin};

#[cw_serde]
pub struct InstantiateMsg {
    pub origin: Option<String>,
    pub owner: Option<String>,
    pub transfer_fee: Option<Coin>,
    pub whitelist: Option<Vec<u64>>,
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
    AddToWhitelist {
        value: u64,
    },
    RemoveFromWhitelist {
        value: u64,
    },
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
