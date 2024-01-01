use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    testing::mock_dependencies, to_json_binary, IbcEndpoint, IbcTimeout, Timestamp,
};

use super::*;

#[cw_serde]
#[derive(Default)]
pub struct IncomingProxyContract {}

impl IncomingProxyExecute for IncomingProxyContract {}
impl IncomingProxyQuery for IncomingProxyContract {}

#[test]
fn test_assert_origin() {
    let mut deps = mock_dependencies();
    let error = IncomingProxyContract::default()
        .assert_origin(deps.as_ref().storage, "sender".to_string())
        .unwrap_err();
    assert_eq!(
        error,
        IncomingProxyError::UnauthorizedOrigin("sender".to_string())
    );

    ORIGIN
        .save(deps.as_mut().storage, &Addr::unchecked("sender"))
        .unwrap();
    IncomingProxyContract::default()
        .assert_origin(deps.as_ref().storage, "sender".to_string())
        .unwrap();
}

#[test]
fn test_assert_packet_data() {
    let mut deps = mock_dependencies();
    let packet = IbcPacket::new(
        to_json_binary(&{}).unwrap(),
        IbcEndpoint {
            port_id: "port-0".to_string(),
            channel_id: "channel-0".to_string(),
        },
        IbcEndpoint {
            port_id: "port-1".to_string(),
            channel_id: "channel-1".to_string(),
        },
        0,
        IbcTimeout::with_timestamp(Timestamp::from_seconds(0)),
    );

    let error = IncomingProxyContract::default()
        .assert_packet_data(deps.as_ref().storage, packet.clone())
        .unwrap_err();
    assert_eq!(
        error,
        IncomingProxyError::UnauthorizedSourceChannel("channel-0".to_string())
    );

    SOURCE_CHANNELS
        .save(
            deps.as_mut().storage,
            "channel-1".to_string(),
            &"channel-1".to_string(),
        )
        .unwrap();
    IncomingProxyContract::default()
        .assert_packet_data(deps.as_ref().storage, packet)
        .unwrap();
}
