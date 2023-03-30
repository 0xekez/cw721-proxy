use cosmwasm_std::{to_binary, Addr, Empty, StdResult};
use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
};

struct Test {
    pub app: App,
    pub cw721s: Vec<Addr>,
    pub minter: Addr,

    pub cw721_proxy: Addr,
    pub mock_receiver: Addr,

    nfts_minted: usize,
}

impl Test {
    pub fn new(cw721s: usize) -> Self {
        let mut app = App::default();
        let minter = Addr::unchecked("minter");

        let cw721_id = app.store_code(cw721_base());
        let cw721_proxy_code_id = app.store_code(cw721_proxy());
        let proxy_tester_code_id = app.store_code(cw721_proxy_tester());

        let mock_receiver = app
            .instantiate_contract(
                proxy_tester_code_id,
                minter.clone(),
                &cw721_proxy_tester::msg::InstantiateMsg::default(),
                &[],
                "proxy_tester",
                None,
            )
            .unwrap();

        let cw721_proxy = app
            .instantiate_contract(
                cw721_proxy_code_id,
                minter.clone(),
                &InstantiateMsg {
                    origin: Some(mock_receiver.to_string()),
                    whitelist: None,
                },
                &[],
                "sender_whitelist",
                None,
            )
            .unwrap();

        let cw721_instantiate_msg = |id: usize| cw721_base::msg::InstantiateMsg {
            name: format!("cw721 {}", id),
            symbol: format!("{}", id),
            minter: minter.to_string(),
        };
        let cw721s: Vec<_> = (0..cw721s)
            .map(|id| {
                app.instantiate_contract(
                    cw721_id,
                    minter.clone(),
                    &cw721_instantiate_msg(id),
                    &[],
                    format!("cw721 {}", id),
                    None,
                )
                .unwrap()
            })
            .collect();

        Self {
            app,
            cw721s,
            minter,
            cw721_proxy,
            mock_receiver,
            nfts_minted: 0,
        }
    }

    pub fn add_to_whitelist(
        &mut self,
        owner: Addr,
        sender: String,
    ) -> Result<AppResponse, anyhow::Error> {
        let res = self.app.execute_contract(
            owner.clone(),
            self.cw721_proxy.clone(),
            &ExecuteMsg::AddToWhitelist { sender },
            &[],
        )?;
        Ok(res)
    }

    pub fn remove_from_whitelist(
        &mut self,
        owner: Addr,
        sender: String,
    ) -> Result<AppResponse, anyhow::Error> {
        let res = self.app.execute_contract(
            owner.clone(),
            self.cw721_proxy.clone(),
            &ExecuteMsg::RemoveFromWhitelist { sender },
            &[],
        )?;
        Ok(res)
    }

    pub fn mint(&mut self, collection: Addr) -> Result<String, anyhow::Error> {
        self.nfts_minted += 1;

        self.app.execute_contract(
            self.minter.clone(),
            collection,
            &cw721_base::msg::ExecuteMsg::<Empty, Empty>::Mint(cw721_base::MintMsg::<Empty> {
                token_id: self.nfts_minted.to_string(),
                owner: self.minter.to_string(),
                token_uri: None,
                extension: Default::default(),
            }),
            &[],
        )?;
        // return token id
        Ok(self.nfts_minted.to_string())
    }

    pub fn send_nft(
        &mut self,
        collection: Addr,
        token_id: String,
    ) -> Result<AppResponse, anyhow::Error> {
        let res = self.app.execute_contract(
            self.minter.clone(),
            collection.clone(),
            &cw721_base::msg::ExecuteMsg::<Empty, Empty>::SendNft {
                contract: self.cw721_proxy.to_string(),
                token_id,
                msg: to_binary("hello")?,
            },
            &[],
        )?;
        Ok(res)
    }

    pub fn query_last_msg(&self) -> StdResult<cw721_proxy_tester::msg::ExecuteMsg> {
        // in case proxy passed message to origin
        self.app.wrap().query_wasm_smart(
            &self.mock_receiver,
            &cw721_proxy_tester::msg::QueryMsg::LastMsg {},
        )
    }
}

fn cw721_proxy() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

fn cw721_base() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw721_base::entry::execute,
        cw721_base::entry::instantiate,
        cw721_base::entry::query,
    );
    Box::new(contract)
}

fn cw721_proxy_tester() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw721_proxy_tester::contract::execute,
        cw721_proxy_tester::contract::instantiate,
        cw721_proxy_tester::contract::query,
    );
    Box::new(contract)
}

#[test]
fn test_origin_specified() {
    let mut app = App::default();
    let cw721_proxy_code_id = app.store_code(cw721_proxy());

    // Check that origin is set to instantiator if origin is None
    // during instantiation.
    let cw721_proxy = app
        .instantiate_contract(
            cw721_proxy_code_id,
            Addr::unchecked("ark_protocol"),
            &InstantiateMsg {
                whitelist: None,
                origin: Some("ark_protocol".to_string()),
            },
            &[],
            "only whitelisted addresses are alllowed",
            None,
        )
        .unwrap();

    let origin: Addr = app
        .wrap()
        .query_wasm_smart(&cw721_proxy, &QueryMsg::Origin {})
        .unwrap();
    assert_eq!(origin, Addr::unchecked("ark_protocol"));

    // assert wl is empty
    let whitelist: Vec<Addr> = app
        .wrap()
        .query_wasm_smart(&cw721_proxy, &QueryMsg::Whitelist {})
        .unwrap();
    assert_eq!(whitelist, Vec::<Addr>::new())
}

#[test]
fn add_to_whitelist_authorized() {
    let mut test = Test::new(1);
    test.add_to_whitelist(test.minter.clone(), "any".to_string())
        .unwrap();
}

#[test]
fn add_to_whitelist_unauthorized() {
    let mut test = Test::new(1);
    let err: ContractError = test
        .add_to_whitelist(Addr::unchecked("unauthorized"), "any".to_string())
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        ContractError::Unauthorized {
            addr: "unauthorized".to_string()
        }
    )
}

#[test]
fn remove_from_whitelist_authorized() {
    let mut test = Test::new(1);
    test.remove_from_whitelist(test.minter.clone(), "any".to_string())
        .unwrap();
}

#[test]
fn remove_from_whitelist_unauthorized() {
    let mut test = Test::new(1);
    let err: ContractError = test
        .remove_from_whitelist(Addr::unchecked("unauthorized"), "any".to_string())
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        ContractError::Unauthorized {
            addr: "unauthorized".to_string()
        }
    )
}

#[test]
fn send_authorized_sender() {
    let mut test = Test::new(1);
    test.add_to_whitelist(test.minter.clone(), test.cw721s[0].to_string())
        .unwrap();

    let token_id = test.mint(test.cw721s[0].clone()).unwrap();
    test.send_nft(test.cw721s[0].clone(), token_id.clone())
        .unwrap();
    match test.query_last_msg().unwrap() {
        cw721_proxy_tester::msg::ExecuteMsg::ReceiveProxyNft { eyeball, msg } => {
            assert_eq!(eyeball, test.cw721s[0].clone());
            assert_eq!(
                msg,
                cw721::Cw721ReceiveMsg {
                    sender: test.minter.to_string(),
                    token_id,
                    msg: to_binary("hello").unwrap()
                }
            )
        }
    }
}

#[test]
fn send_unauthorized_sender() {
    let mut test = Test::new(1);
    let token_id = test.mint(test.cw721s[0].clone()).unwrap();
    let err: ContractError = test
        .send_nft(test.cw721s[0].clone(), token_id)
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        ContractError::Unauthorized {
            addr: test.cw721s[0].clone().to_string()
        }
    )
}
