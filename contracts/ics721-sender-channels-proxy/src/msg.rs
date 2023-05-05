use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, IbcTimeout};

#[cw_serde]
pub struct InstantiateMsg {
    pub origin: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Incoming msg from CW721 contract for ICS721 transfer.
    ReceiveNft(cw721::Cw721ReceiveMsg),
    /// Add CW721 contract and channels to whitelist.
    AddToWhitelist {
        sender: String,
        channels: Vec<String>,
    },
    /// Add CW721 conract and channels to whitelist.
    RemoveFromWhitelist { sender: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
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

/// Copied from: https://github.com/public-awesome/ics721/blob/main/contracts/cw-ics721-bridge/src/msg.rs#L84-L95
#[cw_serde]
pub struct IbcOutgoingMsg {
    /// The address that should receive the NFT being sent on the
    /// *receiving chain*.
    pub receiver: String,
    /// The *local* channel ID this ought to be sent away on. This
    /// contract must have a connection on this channel.
    pub channel_id: String,
    /// Timeout for the IBC message.
    pub timeout: IbcTimeout,
    /// Memo to add custom string to the msg
    pub memo: Option<String>,
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
    },
}
