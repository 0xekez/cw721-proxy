use cosmwasm_std::{coin, to_binary, Addr, Coin, Empty, StdResult};
use cw721_governed_proxy::error::ContractError as GovernedContractError;
use cw721_proxy_multi_test::Test as GovernedMultiTest;
use cw_multi_test::{AppResponse, Contract, ContractWrapper, Executor};

use crate::{
    entry,
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
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
        set_owner: bool,
        whitelist: Option<Vec<String>>,
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
                    whitelist,
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

    pub fn add_to_whitelist(
        &mut self,
        owner: Addr,
        channel: String,
    ) -> Result<AppResponse, anyhow::Error> {
        let res = self.governed_multi_test.app.execute_contract(
            owner.clone(),
            self.proxy.clone(),
            &ExecuteMsg::AddToWhitelist { value: channel },
            &[],
        )?;
        Ok(res)
    }

    pub fn remove_from_whitelist(
        &mut self,
        owner: Addr,
        channel: String,
    ) -> Result<AppResponse, anyhow::Error> {
        let res = self.governed_multi_test.app.execute_contract(
            owner.clone(),
            self.proxy.clone(),
            &ExecuteMsg::RemoveFromWhitelist { value: channel },
            &[],
        )?;
        Ok(res)
    }

    pub fn clear_whitelist(&mut self, owner: Addr) -> Result<AppResponse, anyhow::Error> {
        let res = self.governed_multi_test.app.execute_contract(
            owner.clone(),
            self.proxy.clone(),
            &ExecuteMsg::ClearWhitelist(),
            &[],
        )?;
        Ok(res)
    }

    pub fn query_whitelist(&self) -> StdResult<Vec<String>> {
        // in case proxy passed message to origin
        self.governed_multi_test
            .app
            .wrap()
            .query_wasm_smart(&self.proxy, &QueryMsg::Whitelist {})
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
            &ExecuteMsg::BridgeNft {
                collection: collection.to_string(),
                token_id,
                msg: to_binary(&self.governed_multi_test.ibc_outgoing_msg(channel_id))?,
            },
            &funds,
        )?;

        Ok(res)
    }
}

#[test]
fn add_to_whitelist_authorized() {
    let transfer_fee = Some(coin(100, "uark"));
    let mut test = Test::new(1, transfer_fee, true, None);
    assert_eq!(test.query_whitelist().unwrap(), Vec::<String>::new());
    test.add_to_whitelist(test.governed_multi_test.minter.clone(), "any".to_string())
        .unwrap();
    assert_eq!(test.query_whitelist().unwrap(), vec!["any".to_string()]);
}

#[test]
fn add_to_whitelist_unauthorized() {
    let transfer_fee = Some(coin(100, "uark"));
    let mut test = Test::new(1, transfer_fee, false, None);
    let channel = "any";
    let err: ContractError = test
        .add_to_whitelist(Addr::unchecked("unauthorized"), channel.to_string())
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        ContractError::GovernanceError(GovernedContractError::Unauthorized {
            addr: "unauthorized".to_string()
        })
    )
}

#[test]
fn remove_from_whitelist_authorized() {
    let transfer_fee = Some(coin(100, "uark"));
    let whitelist = vec!["any".to_string()];
    let mut test = Test::new(1, transfer_fee, true, Some(whitelist.clone()));
    assert_eq!(test.query_whitelist().unwrap(), whitelist,);
    test.remove_from_whitelist(test.governed_multi_test.minter.clone(), "any".to_string())
        .unwrap();
    assert_eq!(test.query_whitelist().unwrap(), Vec::<String>::new(),);
}

#[test]
fn remove_from_whitelist_unauthorized() {
    let transfer_fee = Some(coin(100, "uark"));
    let mut test = Test::new(1, transfer_fee, false, None);
    let err: ContractError = test
        .remove_from_whitelist(Addr::unchecked("unauthorized"), "any".to_string())
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        ContractError::GovernanceError(GovernedContractError::Unauthorized {
            addr: "unauthorized".to_string()
        })
    )
}

#[test]
fn clear_whitelist_authorized() {
    let transfer_fee = Some(coin(100, "uark"));
    let whitelist = vec!["any".to_string()];
    let mut test = Test::new(1, transfer_fee, true, Some(whitelist.clone()));
    assert_eq!(test.query_whitelist().unwrap(), whitelist,);

    test.clear_whitelist(test.governed_multi_test.minter.clone())
        .unwrap();
    assert_eq!(test.query_whitelist().unwrap(), Vec::<String>::new(),)
}

#[test]
fn clear_whitelist_unauthorized() {
    let transfer_fee = Some(coin(100, "uark"));
    let mut test = Test::new(1, transfer_fee, false, None);
    let err: ContractError = test
        .clear_whitelist(Addr::unchecked("unauthorized"))
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        ContractError::GovernanceError(GovernedContractError::Unauthorized {
            addr: "unauthorized".to_string()
        })
    )
}

//-- from governed test, test bridge and send nft again, due to new whitelist logic

#[test]
fn bridge_nft_no_transfer_fee_whitelisted() {
    let channel = "any";
    let mut test = Test::new(1, None, true, Some(vec![channel.to_string()]));
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
fn send_nft_no_transfer_fee_whitelisted() {
    let channel = "any";
    let mut test = Test::new(1, None, true, Some(vec![channel.to_string()]));
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

#[test]
fn bridge_nft_no_transfer_fee_not_whitelisted() {
    let mut test = Test::new(1, None, true, None);
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
        ContractError::NotWhitelisted {
            requestee: channel.to_string()
        }
    )
}

#[test]
fn send_nft_no_transfer_fee_not_whitelisted() {
    let mut test = Test::new(1, None, true, None);
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
        ContractError::NotWhitelisted {
            requestee: channel.to_string()
        }
    )
}
// ----
