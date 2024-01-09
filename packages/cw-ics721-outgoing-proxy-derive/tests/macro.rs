use cw_ics721_outgoing_proxy_derive::cw721_receive_nft;

#[cw721_receive_nft]
#[allow(clippy::large_enum_variant)]
enum ExecuteMsg {
    Foo,
}

#[test]
fn derive_execute_variants() {
    let msg = ExecuteMsg::Foo;

    // If this compiles we have won.
    match msg {
        ExecuteMsg::ReceiveNft { .. } | ExecuteMsg::Foo => "yay",
    };
}
