use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, IbcTimeout};

#[cw_serde]
pub struct InstantiateMsg {
    pub origin: Option<String>,
    pub whitelist: Option<Vec<String>>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Incoming msg from CW721 contract for ICS721 transfer.
    ReceiveNft(cw721::Cw721ReceiveMsg),
    /// Add channel to whitelist.
    AddToWhitelist {
        channel: String,
    },
    /// Remove channel from whitelist.
    RemoveFromWhitelist {
        channel: String,
    },
    Origin(Addr),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Addr)]
    Owner {},

    /// Gets ICS721 contract.
    #[returns(Addr)]
    Origin {},

    /// Gets a list of channels authorized for ICS721 transfers.
    #[returns(Vec<String>)]
    Whitelist {},

    /// True in case channel is authorized for ICS721 transfers.
    #[returns(bool)]
    WhiteListed { channel: String },
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

#[cw_serde]
pub enum MigrateMsg {
    WithUpdate {
        whitelist: Option<Vec<String>>,
        origin: Option<String>,
    },
}
