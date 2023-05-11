use cosmwasm_schema::cw_serde;
use cosmwasm_std::IbcTimeout;

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
