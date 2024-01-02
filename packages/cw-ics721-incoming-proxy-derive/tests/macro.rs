use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_ics721_incoming_proxy_derive::{ics721_incoming_proxy_execute, ics721_incoming_proxy_query};

#[ics721_incoming_proxy_execute]
#[cw_serde]
#[allow(clippy::large_enum_variant)]
enum ExecuteMsg {
    Foo,
}

#[ics721_incoming_proxy_query]
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
        ExecuteMsg::Ics721ReceivePacketMsg { .. } | ExecuteMsg::Foo => "yay",
    };
}

#[test]
fn derive_query_variants() {
    let msg = QueryMsg::Foo;

    // If this compiles we have won.
    match msg {
        QueryMsg::GetOrigin {}
        | QueryMsg::GetChannels { .. }
        | QueryMsg::Foo
        | QueryMsg::Bar(_)
        | QueryMsg::Fuzz { .. } => "yay",
    };
}
