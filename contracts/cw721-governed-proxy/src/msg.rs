use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Coin};

#[cw_serde]
pub struct InstantiateMsg {
    pub origin: Option<String>,
    pub owner: Option<String>,
    pub transfer_fee: Option<Coin>,
}

#[cw_serde]
pub enum ExecuteMsg {
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
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Addr)]
    Owner {},

    /// Gets ICS721 contract.
    #[returns(Addr)]
    Origin {},

    #[returns(Option<Coin>)]
    TransferFee {},
}

#[cw_serde]
pub enum MigrateMsg {
    WithUpdate {
        origin: Option<String>,
        transfer_fee: Option<Coin>,
    },
}
