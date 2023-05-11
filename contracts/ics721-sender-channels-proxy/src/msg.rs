use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub origin: Option<String>,
    pub whitelist: Option<Vec<SenderToChannelsResponse>>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Owner(String),
    /// Incoming msg from CW721 contract for ICS721 transfer.
    ReceiveNft(cw721::Cw721ReceiveMsg),
    /// Add CW721 contract and channels to whitelist.
    AddToWhitelist {
        sender: String,
        channels: Vec<String>,
    },
    /// Add CW721 conract and channels to whitelist.
    RemoveFromWhitelist {
        sender: String,
    },
    Origin(String),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Addr)]
    Owner {},

    /// Gets ICS721 contract.
    #[returns(Addr)]
    Origin {},

    /// Gets a list of CW721 contract and channels authorized for ICS721 transfers.
    #[returns(Vec<SenderToChannelsResponse>)]
    Whitelist {},

    /// True in case CW721 contract and channel is authorized for ICS721 transfers.
    #[returns(bool)]
    WhiteListed { sender: String, channel: String },
}

/// Authorized CW721 contract and channel for ICS721 transfers.
#[cw_serde]
pub struct SenderToChannelsResponse {
    /// Authorized CW721 contract for ICS721 transfers.
    pub sender: String,
    /// Authorized channels for ICS721 transfers.
    pub channels: Vec<String>,
}

#[cw_serde]
pub enum MigrateMsg {
    WithUpdate {
        whitelist: Option<Vec<SenderToChannelsResponse>>,
        origin: Option<String>,
    },
}
