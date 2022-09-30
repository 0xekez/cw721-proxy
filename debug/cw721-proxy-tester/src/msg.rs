use cosmwasm_schema::cw_serde;
pub use cw721_proxy::ProxyExecuteMsg as ExecuteMsg;

#[cw_serde]
#[derive(Default)]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum QueryMsg {
    /// Gets the last message this contract received.
    LastMsg {},
}
