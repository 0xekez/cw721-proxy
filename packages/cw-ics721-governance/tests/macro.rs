use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_ics721_governance::{cw_ics721_governance_execute, cw_ics721_governance_query, Action};

#[cw_ics721_governance_execute]
#[cw_serde]
enum ExecuteMsg {
    Foo,
    Bar(u64),
    Fuzz { buzz: String },
}

#[cw_ics721_governance_query]
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
        ExecuteMsg::Governance(Action::Owner { .. })
        | ExecuteMsg::Governance(Action::Origin(..))
        | ExecuteMsg::Governance(Action::TransferFee(..))
        | ExecuteMsg::Governance(Action::SendFunds {
            to_address: _,
            amount: _,
        })
        | ExecuteMsg::Governance(Action::BridgeNft {
            collection: _,
            token_id: _,
            msg: _,
        })
        | ExecuteMsg::ReceiveNft(..)
        | ExecuteMsg::Foo
        | ExecuteMsg::Bar(_)
        | ExecuteMsg::Fuzz { .. } => "yay",
    };
}

#[test]
fn derive_query_variants() {
    let msg = QueryMsg::Foo;

    // If this compiles we have won.
    match msg {
        QueryMsg::Governance() | QueryMsg::Foo | QueryMsg::Bar(_) | QueryMsg::Fuzz { .. } => "yay",
    };
}
