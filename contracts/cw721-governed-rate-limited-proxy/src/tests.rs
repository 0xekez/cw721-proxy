use cosmwasm_std::{coin, to_binary, Addr, Coin, Empty};
use cw721_proxy_multi_test::Test as GovernedMultiTest;
use cw_ics721_governance::{Action, GovernanceError};
use cw_multi_test::{AppResponse, Contract, ContractWrapper, Executor};
use cw_rate_limiter::Rate;

use crate::{
    entry,
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg},
};

fn proxy_code() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(entry::execute, entry::instantiate, entry::query);
    Box::new(contract)
}

pub struct Test {
    pub governed_multi_test: GovernedMultiTest,
    pub proxy_code_id: u64,
    pub proxy: Addr,
}

impl Test {
    pub fn new(
        cw721s: usize,
        transfer_fee: Option<Coin>,
        rate_limit: Rate,
        set_owner: bool,
    ) -> Self {
        let mut governed_multi_test = GovernedMultiTest::new(cw721s, transfer_fee);
        let proxy_code_id = governed_multi_test.app.store_code(proxy_code());
        let owner = match set_owner {
            true => Some(governed_multi_test.minter.to_string()),
            false => None,
        };
        let proxy = governed_multi_test
            .app
            .instantiate_contract(
                proxy_code_id,
                governed_multi_test.minter.clone(),
                &InstantiateMsg {
                    origin: Some(governed_multi_test.mock_receiver.to_string()),
                    owner,
                    transfer_fee: governed_multi_test.transfer_fee.clone(),
                    rate_limit,
                },
                &[],
                "governed proxy",
                None,
            )
            .unwrap();
        Self {
            governed_multi_test,
            proxy_code_id,
            proxy,
        }
    }

    pub fn execute_rate_limit(
        &mut self,
        owner: Addr,
        rate_limit: Rate,
    ) -> Result<AppResponse, anyhow::Error> {
        let res = self.governed_multi_test.app.execute_contract(
            owner.clone(),
            self.proxy.clone(),
            &ExecuteMsg::RateLimit(rate_limit),
            &[],
        )?;
        Ok(res)
    }

    pub fn bridge_nft(
        &mut self,
        sender: Addr,
        proxy: Addr,
        collection: Addr,
        token_id: String,
        channel_id: String,
        transfer_fee: Option<Coin>,
    ) -> Result<AppResponse, anyhow::Error> {
        let funds = transfer_fee.map_or(vec![], |fee| vec![fee]);
        let res = self.governed_multi_test.app.execute_contract(
            sender,
            proxy,
            &ExecuteMsg::Governance(Action::BridgeNft {
                collection: collection.to_string(),
                token_id,
                msg: to_binary(&self.governed_multi_test.ibc_outgoing_msg(channel_id))?,
            }),
            &funds,
        )?;

        Ok(res)
    }
}

#[test]
fn rate_limit_is_zero() {
    let transfer_fee = Some(coin(100, "uark"));
    let mut test = Test::new(1, transfer_fee, Rate::Blocks(1), true);
    let err: ContractError = test
        .execute_rate_limit(test.governed_multi_test.minter.clone(), Rate::PerBlock(0))
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, ContractError::ZeroRate {})
}

#[test]
fn rate_limit_owner() {
    let transfer_fee = Some(coin(100, "uark"));
    let mut test = Test::new(1, transfer_fee, Rate::Blocks(1), true);
    test.execute_rate_limit(test.governed_multi_test.minter.clone(), Rate::Blocks(1))
        .unwrap();
}

#[test]
fn rate_limit_no_owner() {
    let transfer_fee = Some(coin(100, "uark"));
    let mut test = Test::new(1, transfer_fee, Rate::Blocks(1), false);
    let err: ContractError = test
        .execute_rate_limit(Addr::unchecked("unauthorized"), Rate::Blocks(1))
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, ContractError::Governance(GovernanceError::NoOwner))
}

#[test]
fn rate_limit_not_owner() {
    let transfer_fee = Some(coin(100, "uark"));
    let mut test = Test::new(1, transfer_fee, Rate::Blocks(1), true);
    let err: ContractError = test
        .execute_rate_limit(Addr::unchecked("unauthorized"), Rate::Blocks(1))
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        ContractError::Governance(GovernanceError::NotOwner("unauthorized".to_string()))
    )
}

//-- from governed test, test bridge and send nft again, due to new whitelist logic

#[test]
fn bridge_nft_no_transfer_fee() {
    let mut test = Test::new(1, None, Rate::Blocks(1), true);
    let channel = "any";
    let token_id = test
        .governed_multi_test
        .mint(test.governed_multi_test.cw721s[0].clone())
        .unwrap();
    test.governed_multi_test
        .approve(
            test.governed_multi_test.cw721s[0].clone(),
            token_id.clone(),
            test.proxy.to_string(),
        )
        .unwrap();

    test.bridge_nft(
        test.governed_multi_test.minter.clone(),
        test.proxy.clone(),
        test.governed_multi_test.cw721s[0].clone(),
        token_id.clone(),
        channel.to_string(),
        test.governed_multi_test.transfer_fee.clone(),
    )
    .unwrap();
    match test.governed_multi_test.query_last_msg().unwrap() {
        cw721_proxy_tester::msg::ExecuteMsg::ReceiveProxyNft { eyeball, msg } => {
            assert_eq!(eyeball, test.governed_multi_test.cw721s[0].clone());
            assert_eq!(
                msg,
                cw721::Cw721ReceiveMsg {
                    sender: test.governed_multi_test.minter.to_string(),
                    token_id,
                    msg: to_binary(
                        &test
                            .governed_multi_test
                            .ibc_outgoing_msg(channel.to_string())
                    )
                    .unwrap(),
                }
            )
        }
    }
}

#[test]
fn send_nft_no_transfer_fee() {
    let mut test = Test::new(1, None, Rate::Blocks(1), true);
    let channel = "any";
    let token_id = test
        .governed_multi_test
        .mint(test.governed_multi_test.cw721s[0].clone())
        .unwrap();

    test.governed_multi_test
        .send_nft(
            test.governed_multi_test.minter.clone(),
            test.proxy.to_string(),
            test.governed_multi_test.cw721s[0].clone(),
            token_id.clone(),
            channel.to_string(),
            None,
        )
        .unwrap();
    match test.governed_multi_test.query_last_msg().unwrap() {
        cw721_proxy_tester::msg::ExecuteMsg::ReceiveProxyNft { eyeball, msg } => {
            assert_eq!(eyeball, test.governed_multi_test.cw721s[0].clone());
            assert_eq!(
                msg,
                cw721::Cw721ReceiveMsg {
                    sender: test.governed_multi_test.minter.to_string(),
                    token_id,
                    msg: to_binary(&test.governed_multi_test.ibc_outgoing_msg("any".to_string()))
                        .unwrap(),
                }
            )
        }
    }
}
// ----
