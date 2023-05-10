use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub origin: Option<String>,
    pub whitelist: Option<Vec<String>>,
}

#[cw_serde]
pub enum ExecuteMsg {
    ReceiveNft(cw721::Cw721ReceiveMsg),
    /// Add CW721 contract to whitelist.
    AddToWhitelist {
        sender: String,
    },
    /// Add CW721 contract to whitelist.
    RemoveFromWhitelist {
        sender: String,
    },
    Origin(Addr),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Gets ICS721 contract.
    #[returns(Addr)]
    Origin {},

    /// Gets a list of CW721 contracts authorized for ICS721 transfers.
    #[returns(Vec<Addr>)]
    Whitelist {},

    /// True in case CW721 contract is authorized for ICS721 transfers.
    #[returns(bool)]
    WhiteListed { sender: String },
}

#[cw_serde]
pub enum MigrateMsg {
    WithUpdate {
        whitelist: Option<Vec<String>>,
        origin: Option<String>,
    },
}
