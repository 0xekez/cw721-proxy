use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{
    coin, to_json_binary, Addr, Coin, Empty, IbcEndpoint, IbcPacket, IbcTimeout, StdError,
    StdResult, Timestamp,
};
use cw_incoming_proxy::{IncomingProxyError, IncomingProxyExecute, IncomingProxyQuery};
use cw_incoming_proxy_derive::{cw_incoming_proxy_execute, cw_incoming_proxy_query};
use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};
use ics721_types::{ibc_types::NonFungibleTokenPacketData, token_types::ClassId};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

pub const ORIGIN_ADDR: &str = "ics721_address";
pub const SENDER_ADDR: &str = "sender_address";
pub const OTHER_ADDR: &str = "other_address";
pub const CHANNEL_1: &str = "channel_1";

#[cw_serde]
pub struct MockInstantiateMsg {
    origin: Option<String>,
    source_channels: Option<Vec<String>>,
}

#[cw_incoming_proxy_execute]
#[cw_serde]
pub enum MockExecuteMsg {}

#[cw_incoming_proxy_query]
#[derive(QueryResponses)]
#[cw_serde]
pub enum MockQueryMsg {}

#[cw_serde]
pub enum MockMigrateMsg {
    WithUpdate {
        ics721: Option<String>,
        source_channels: Option<Vec<String>>,
    },
}

#[cw_serde]
#[derive(Default)]
pub struct IncomingProxyContract {}

impl IncomingProxyExecute for IncomingProxyContract {}
impl IncomingProxyQuery for IncomingProxyContract {}

#[derive(Error, Debug, PartialEq)]
pub enum MockContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    IncomingProxyError(#[from] IncomingProxyError),
}

pub mod entry {
    use super::*;
    use cosmwasm_std::{
        to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    };
    use cw_incoming_proxy::IncomingProxyExecute;

    pub fn instantiate(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: MockInstantiateMsg,
    ) -> StdResult<Response> {
        IncomingProxyContract::default().initialize(
            deps.storage,
            deps.api,
            msg.origin,
            msg.source_channels,
        )?;
        Ok(Response::default())
    }

    pub fn execute(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: MockExecuteMsg,
    ) -> Result<Response, MockContractError> {
        match msg {
            MockExecuteMsg::Ics721ReceivePacketMsg { packet, data } => {
                IncomingProxyContract::default().execute_ics721_receive_packet_msg::<Empty>(
                    deps.storage,
                    &info,
                    packet,
                    data,
                )?;
                Ok(Response::default()
                    .add_attribute("method", "execute")
                    .add_attribute("action", "ics721_receive_packet_msg"))
            }
        }
    }

    pub fn query(deps: Deps, _env: Env, msg: MockQueryMsg) -> StdResult<Binary> {
        match msg {
            MockQueryMsg::GetOrigin {} => {
                to_json_binary(&IncomingProxyContract::default().get_origin(deps.storage)?)
            }
            MockQueryMsg::GetSourceChannels { limit, start_after } => to_json_binary(
                &IncomingProxyContract::default().get_source_channels(deps, start_after, limit)?,
            ),
        }
    }

    pub fn migrate(deps: DepsMut, _env: Env, msg: MockMigrateMsg) -> StdResult<Response> {
        match msg {
            MockMigrateMsg::WithUpdate {
                ics721,
                source_channels,
            } => {
                IncomingProxyContract::default().initialize(
                    deps.storage,
                    deps.api,
                    ics721,
                    source_channels,
                )?;
                Ok(Response::default())
            }
        }
    }
}

pub fn contract_code() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(entry::execute, entry::instantiate, entry::query)
        .with_migrate(entry::migrate);
    Box::new(contract)
}

pub struct Ics721IncomingProxyMultiTest {
    pub app: App,
    pub sender: Addr,
    pub other: Addr,
    pub origin: Addr,
    pub source_channels: Option<Vec<String>>,
    pub code_id: u64,
    pub contract: Addr,
}

impl Ics721IncomingProxyMultiTest {
    pub fn new(origin: String, source_channels: Option<Vec<String>>) -> Self {
        let sender = Addr::unchecked(SENDER_ADDR);
        let other = Addr::unchecked(OTHER_ADDR);
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &sender, vec![coin(20000, "uark")])
                .unwrap();
            router
                .bank
                .init_balance(storage, &other, vec![coin(10000, "uark")])
                .unwrap();
        });
        let code_id = app.store_code(contract_code());
        let contract = app
            .instantiate_contract(
                code_id,
                sender.clone(),
                &MockInstantiateMsg {
                    source_channels: source_channels.clone(),
                    origin: Some(origin.clone()),
                },
                &[],
                "contract",
                None,
            )
            .unwrap();

        Self {
            app,
            sender,
            other,
            origin: Addr::unchecked(origin),
            source_channels,
            code_id,
            contract,
        }
    }

    pub fn query_wasm_smart<T: DeserializeOwned>(
        &self,
        contract_addr: impl Into<String>,
        msg: &impl Serialize,
    ) -> StdResult<T> {
        self.app.wrap().query_wasm_smart(contract_addr, msg)
    }

    fn query_source_channels(&self) -> StdResult<Vec<String>> {
        // in case proxy passed message to origin
        self.query_wasm_smart(
            &self.contract,
            &MockQueryMsg::GetSourceChannels {
                start_after: None,
                limit: None,
            },
        )
    }

    fn query_origin(&self) -> StdResult<Addr> {
        // in case proxy passed message to origin
        self.query_wasm_smart(&self.contract, &MockQueryMsg::GetOrigin {})
    }

    fn execute_contract<T: Serialize + std::fmt::Debug>(
        &mut self,
        sender: Addr,
        contract_addr: Addr,
        msg: &T,
        send_funds: &[Coin],
    ) -> Result<AppResponse, anyhow::Error> {
        self.app
            .execute_contract(sender, contract_addr, msg, send_funds)
    }

    fn execute_ics721_receive_packet(
        &mut self,
        sender: Addr,
        packet: cosmwasm_std::IbcPacket,
        data: ics721_types::ibc_types::NonFungibleTokenPacketData,
        send_funds: &[Coin],
    ) -> Result<AppResponse, anyhow::Error> {
        self.execute_contract(
            sender,
            self.contract.clone(),
            &MockExecuteMsg::Ics721ReceivePacketMsg { packet, data },
            send_funds,
        )
    }
}

#[test]
fn test_instantiate() {
    // no source channels defined
    {
        let test = Ics721IncomingProxyMultiTest::new(ORIGIN_ADDR.to_string(), None);
        let origin = test.query_origin().unwrap();
        assert_eq!(origin, Addr::unchecked(ORIGIN_ADDR));
    }

    // source channels defined
    {
        let source_channels = vec!["channel-0".to_string(), "channel-1".to_string()];
        let test = Ics721IncomingProxyMultiTest::new(
            ORIGIN_ADDR.to_string(),
            Some(source_channels.clone()),
        );
        let source_channels = test.query_source_channels().unwrap();
        assert_eq!(source_channels, source_channels);
    }
}

#[test]
fn test_ics721_receive_packet() {
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

    let data = NonFungibleTokenPacketData {
        class_id: ClassId::new("some/class/id"),
        sender: SENDER_ADDR.to_string(),
        receiver: "receiver".to_string(),
        class_data: None,
        class_uri: None,
        memo: None,
        token_data: None,
        token_ids: vec![],
        token_uris: None,
    };

    // test unauthorized
    {
        let mut test = Ics721IncomingProxyMultiTest::new(ORIGIN_ADDR.to_string(), None);
        let error: MockContractError = test
            .execute_ics721_receive_packet(
                Addr::unchecked(SENDER_ADDR),
                packet.clone(),
                data.clone(),
                &[],
            )
            .unwrap_err()
            .downcast()
            .unwrap();

        assert_eq!(
            error,
            MockContractError::IncomingProxyError(IncomingProxyError::UnauthorizedOrigin(
                SENDER_ADDR.to_string()
            ))
        );

        let error: MockContractError = test
            .execute_ics721_receive_packet(
                Addr::unchecked(ORIGIN_ADDR),
                packet.clone(),
                data.clone(),
                &[],
            )
            .unwrap_err()
            .downcast()
            .unwrap();

        assert_eq!(
            error,
            MockContractError::IncomingProxyError(IncomingProxyError::UnauthorizedSourceChannel(
                "channel-0".to_string()
            ))
        );
    }

    // test authorized
    {
        let source_channels = vec!["channel-0".to_string()];
        let mut test =
            Ics721IncomingProxyMultiTest::new(ORIGIN_ADDR.to_string(), Some(source_channels));
        test.execute_ics721_receive_packet(
            Addr::unchecked(ORIGIN_ADDR),
            packet.clone(),
            data.clone(),
            &[],
        )
        .unwrap();
    }
}
