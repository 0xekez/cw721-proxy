use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub origin: Option<String>,
    pub whitelist: Option<Vec<u64>>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Owner(String),
    ReceiveNft(cw721::Cw721ReceiveMsg),
    AddToWhitelist { code_id: u64 },
    RemoveFromWhitelist { code_id: u64 },
    Origin(String),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Addr)]
    Owner {},

    #[returns(Addr)]
    Origin {},

    #[returns(Vec<Addr>)]
    Whitelist {},

    #[returns(bool)]
    WhiteListed { code_id: u64 },
}

#[cw_serde]
pub enum MigrateMsg {
    WithUpdate {
        whitelist: Option<Vec<u64>>,
        origin: Option<String>,
    },
}
