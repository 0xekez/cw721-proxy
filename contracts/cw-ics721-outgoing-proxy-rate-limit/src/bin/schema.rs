use cosmwasm_schema::write_api;

use cw_ics721_outgoing_proxy_rate_limit::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
