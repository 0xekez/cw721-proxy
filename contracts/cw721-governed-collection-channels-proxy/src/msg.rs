use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Coin};

#[cw_serde]
pub struct InstantiateMsg {
    pub origin: Option<String>,
    pub owner: Option<String>,
    pub transfer_fee: Option<Coin>,
    pub whitelist: Option<Vec<(String, Vec<String>)>>,
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
        collection: String,
        channels: Vec<String>,
    },
    RemoveFromWhitelist {
        collection: String,
    },
    ClearWhitelist(),
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
