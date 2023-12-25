use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_incoming_proxy_derive::{cw_incoming_proxy_execute, cw_incoming_proxy_query};

#[cw_incoming_proxy_execute]
#[cw_serde]
#[allow(clippy::large_enum_variant)]
enum ExecuteMsg {
    Foo,
}

#[cw_incoming_proxy_query]
#[cw_serde]
#[derive(QueryResponses)]
enum QueryMsg {
    #[returns(String)]
    Foo,

    #[returns(String)]
    Bar(u64),

    #[returns(String)]
    Fuzz { buzz: String },
}

#[test]
fn derive_execute_variants() {
    let msg = ExecuteMsg::Foo;

    // If this compiles we have won.
    match msg {
        ExecuteMsg::Ics721ReceivePacketMsg { .. }
        | ExecuteMsg::Foo => "yay",
    };
}

#[test]
fn derive_query_variants() {
    let msg = QueryMsg::Foo;

    // If this compiles we have won.
    match msg {
        QueryMsg::GetOrigin {}
        | QueryMsg::GetSourceChannels { .. }
        | QueryMsg::Foo
        | QueryMsg::Bar(_)
        | QueryMsg::Fuzz { .. } => "yay",
    };
}
