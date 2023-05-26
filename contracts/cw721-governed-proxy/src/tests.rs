use cosmwasm_std::{coin, to_binary, Addr, Coin, Empty};
use cw721_proxy_multi_test::Test as GovernedMultiTest;
use cw_multi_test::{Contract, ContractWrapper, Executor};

use crate::{entry, error::ContractError, msg::InstantiateMsg};

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
    pub fn new(cw721s: usize, transfer_fee: Option<Coin>, set_owner: bool) -> Self {
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
}

#[test]
fn bridge_nft_no_transfer_fee() {
    let mut test = Test::new(1, None, false);
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

    let channel = "any";
    test.governed_multi_test
        .bridge_nft(
            test.governed_multi_test.minter.clone(),
            test.proxy,
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
    let mut test = Test::new(1, None, false);
    let token_id = test
        .governed_multi_test
        .mint(test.governed_multi_test.cw721s[0].clone())
        .unwrap();

    let channel = "any";
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

#[test]
fn bridge_nft_unapproved() {
    let mut test = Test::new(1, None, false);
    let token_id = test
        .governed_multi_test
        .mint(test.governed_multi_test.cw721s[0].clone())
        .unwrap();

    let channel = "any";
    let err: ContractError = test
        .governed_multi_test
        .bridge_nft(
            test.governed_multi_test.minter.clone(),
            test.proxy.clone(),
            test.governed_multi_test.cw721s[0].clone(),
            token_id.clone(),
            channel.to_string(),
            test.governed_multi_test.transfer_fee.clone(),
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        ContractError::Governance(cw_ics721_governance::GovernanceError::MissingApproval {
            spender: test.proxy.to_string(),
            collection: test.governed_multi_test.cw721s[0].to_string(),
            token: token_id,
        })
    )
}

#[test]
fn bridge_nft_no_payment() {
    let transfer_fee = coin(100, "uark");
    let mut test = Test::new(1, Some(transfer_fee.clone()), false);
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

    let channel = "any";
    let err: ContractError = test
        .governed_multi_test
        .bridge_nft(
            test.governed_multi_test.minter.clone(),
            test.proxy,
            test.governed_multi_test.cw721s[0].clone(),
            token_id.clone(),
            channel.to_string(),
            None,
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        ContractError::Governance(
            cw_ics721_governance::GovernanceError::IncorrectPaymentAmount(
                coin(0, "uark"),
                transfer_fee
            )
        )
    )
}

#[test]
fn send_nft_no_payment() {
    let transfer_fee = coin(100, "uark");
    let mut test = Test::new(1, Some(transfer_fee.clone()), false);
    let token_id = test
        .governed_multi_test
        .mint(test.governed_multi_test.cw721s[0].clone())
        .unwrap();

    let channel = "any";
    let err: ContractError = test
        .governed_multi_test
        .send_nft(
            test.governed_multi_test.minter.clone(),
            test.proxy.to_string(),
            test.governed_multi_test.cw721s[0].clone(),
            token_id.clone(),
            channel.to_string(),
            None,
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        ContractError::Governance(
            cw_ics721_governance::GovernanceError::IncorrectPaymentAmount(
                coin(0, "uark"),
                transfer_fee
            )
        )
    )
}

#[test]
fn bridge_nft_correct_payment() {
    let transfer_fee = coin(100, "uark");
    let mut test = Test::new(1, Some(transfer_fee.clone()), false);
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

    let channel = "any";
    test.governed_multi_test
        .bridge_nft(
            test.governed_multi_test.minter.clone(),
            test.proxy,
            test.governed_multi_test.cw721s[0].clone(),
            token_id.clone(),
            channel.to_string(),
            test.governed_multi_test.transfer_fee.clone(),
        )
        .unwrap();
}

#[test]
fn send_nft_correct_payment() {
    let transfer_fee = coin(100, "uark");
    let mut test = Test::new(1, Some(transfer_fee.clone()), false);
    let token_id = test
        .governed_multi_test
        .mint(test.governed_multi_test.cw721s[0].clone())
        .unwrap();

    let channel = "any";
    let err: ContractError = test
        .governed_multi_test
        .send_nft(
            test.governed_multi_test.minter.clone(),
            test.proxy.to_string(),
            test.governed_multi_test.cw721s[0].clone(),
            token_id.clone(),
            channel.to_string(),
            test.governed_multi_test.transfer_fee.clone(), // paid to collection, but proxy needs it!
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        ContractError::Governance(
            cw_ics721_governance::GovernanceError::IncorrectPaymentAmount(
                coin(0, "uark"),
                transfer_fee
            )
        ) // proxy receive 0 coins
    )
}

#[test]
fn bridge_nft_insufficient_payment() {
    let transfer_fee = coin(100, "uark");
    let mut test = Test::new(1, Some(transfer_fee.clone()), false);
    let token_id = test
        .governed_multi_test
        .mint(test.governed_multi_test.cw721s[0].clone())
        .unwrap();

    let channel = "any";
    let err: ContractError = test
        .governed_multi_test
        .bridge_nft(
            test.governed_multi_test.minter.clone(),
            test.proxy,
            test.governed_multi_test.cw721s[0].clone(),
            token_id.clone(),
            channel.to_string(),
            Some(coin(50, "uark")),
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        ContractError::Governance(
            cw_ics721_governance::GovernanceError::IncorrectPaymentAmount(
                coin(50, "uark"),
                transfer_fee
            )
        )
    )
}

#[test]
fn send_nft_insufficient_payment() {
    let transfer_fee = coin(100, "uark");
    let mut test = Test::new(1, Some(transfer_fee.clone()), false);
    let token_id = test
        .governed_multi_test
        .mint(test.governed_multi_test.cw721s[0].clone())
        .unwrap();

    let channel = "any";
    let err: ContractError = test
        .governed_multi_test
        .send_nft(
            test.governed_multi_test.minter.clone(),
            test.proxy.to_string(),
            test.governed_multi_test.cw721s[0].clone(),
            token_id.clone(),
            channel.to_string(),
            Some(coin(50, "uark")), // paid to collection, but proxy needs it!
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        ContractError::Governance(
            cw_ics721_governance::GovernanceError::IncorrectPaymentAmount(
                coin(0, "uark"),
                transfer_fee
            )
        ) // proxy receive 0 coins
    )
}

#[test]
fn bridge_nft_higher_payment() {
    let transfer_fee = coin(100, "uark");
    let mut test = Test::new(1, Some(transfer_fee.clone()), false);
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

    let channel = "any";
    let err: ContractError = test
        .governed_multi_test
        .bridge_nft(
            test.governed_multi_test.minter.clone(),
            test.proxy,
            test.governed_multi_test.cw721s[0].clone(),
            token_id.clone(),
            channel.to_string(),
            Some(coin(150, "uark")),
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        ContractError::Governance(
            cw_ics721_governance::GovernanceError::IncorrectPaymentAmount(
                coin(150, "uark"),
                transfer_fee
            )
        )
    )
}

#[test]
fn send_nft_higher_payment() {
    let transfer_fee = coin(100, "uark");
    let mut test = Test::new(1, Some(transfer_fee.clone()), false);
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

    let channel = "any";
    let err: ContractError = test
        .governed_multi_test
        .send_nft(
            test.governed_multi_test.minter.clone(),
            test.proxy.to_string(),
            test.governed_multi_test.cw721s[0].clone(),
            token_id.clone(),
            channel.to_string(),
            Some(coin(150, "uark")), // paid to collection, but proxy needs it!
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        ContractError::Governance(
            cw_ics721_governance::GovernanceError::IncorrectPaymentAmount(
                coin(0, "uark"),
                transfer_fee
            )
        ) // proxy receive 0 coins
    )
}
