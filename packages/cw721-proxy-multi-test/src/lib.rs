use cosmwasm_schema::cw_serde;
use cosmwasm_std::{coin, to_binary, Addr, Coin, Empty, IbcTimeout, IbcTimeoutBlock, StdResult};
use cw721_base::ExecuteMsg as Cw721ExecuteMsg;
use cw_ics721_governance::{cw_ics721_governance_execute, Action};
use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};
use ibc_outgoing_msg::IbcOutgoingMsg;

#[cw_ics721_governance_execute]
#[cw_serde]
pub enum ExecuteMsg {}

pub struct Test {
    pub app: App,
    #[allow(dead_code)]
    pub cw721_id: u64,
    pub cw721s: Vec<Addr>,
    pub minter: Addr,
    pub other: Addr,

    pub mock_receiver: Addr,
    pub transfer_fee: Option<Coin>,

    pub nfts_minted: usize,
}

impl Test {
    pub fn new(cw721s: usize, transfer_fee: Option<Coin>) -> Self {
        let minter = Addr::unchecked("minter");
        let other = Addr::unchecked("other");
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &minter, vec![coin(10000, "uark")])
                .unwrap();
            router
                .bank
                .init_balance(storage, &other, vec![coin(10000, "uark")])
                .unwrap();
        });

        let cw721_id = app.store_code(cw721_base());
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
            cw721_id,
            cw721s,
            minter,
            other,
            mock_receiver,
            transfer_fee,
            nfts_minted: 0,
        }
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

    pub fn ibc_outgoing_msg(&self, channel_id: String) -> IbcOutgoingMsg {
        IbcOutgoingMsg {
            channel_id,
            memo: None,
            receiver: "dummy".to_string(),
            timeout: IbcTimeout::with_block(IbcTimeoutBlock {
                revision: 0,
                height: 10,
            }),
        }
    }

    pub fn approve(
        &mut self,
        collection: Addr,
        token_id: String,
        spender: String,
    ) -> Result<AppResponse, anyhow::Error> {
        let approve_msg: Cw721ExecuteMsg<Empty, Empty> = Cw721ExecuteMsg::Approve {
            spender,
            token_id,
            expires: Some(cw_utils::Expiration::Never {}),
        };
        let res = self
            .app
            .execute_contract(self.minter.clone(), collection, &approve_msg, &[])?;

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
        let res = self.app.execute_contract(
            sender,
            proxy,
            &ExecuteMsg::Governance(Action::BridgeNft {
                collection: collection.to_string(),
                token_id,
                msg: to_binary(&self.ibc_outgoing_msg(channel_id))?,
            }),
            &funds,
        )?;

        Ok(res)
    }

    pub fn send_nft(
        &mut self,
        sender: Addr,
        contract: String, // target
        collection: Addr, // source
        token_id: String,
        channel_id: String,
        transfer_fee: Option<Coin>,
    ) -> Result<AppResponse, anyhow::Error> {
        let funds = transfer_fee.map_or(vec![], |fee| vec![fee]);
        let res = self.app.execute_contract(
            sender,
            collection,
            &cw721_base::msg::ExecuteMsg::<Empty, Empty>::SendNft {
                contract,
                token_id,
                msg: to_binary(&self.ibc_outgoing_msg(channel_id))?,
            },
            &funds,
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

    pub fn execute_owner(
        &mut self,
        sender: Addr,
        proxy: Addr,
        addr: Addr,
    ) -> Result<AppResponse, anyhow::Error> {
        let res = self.app.execute_contract(
            sender,
            proxy,
            &ExecuteMsg::Governance(Action::Owner(addr.to_string())),
            &[],
        )?;

        Ok(res)
    }

    pub fn execute_origin(
        &mut self,
        sender: Addr,
        proxy: Addr,
        addr: Addr,
    ) -> Result<AppResponse, anyhow::Error> {
        let res = self.app.execute_contract(
            sender,
            proxy,
            &ExecuteMsg::Governance(Action::Origin(addr.to_string())),
            &[],
        )?;

        Ok(res)
    }

    pub fn send_funds(
        &mut self,
        sender: Addr,
        proxy: Addr,
        to_address: String,
        amount: Coin,
    ) -> Result<AppResponse, anyhow::Error> {
        let res = self.app.execute_contract(
            sender,
            proxy,
            &ExecuteMsg::Governance(Action::SendFunds {
                to_address,
                amount: amount.clone(),
            }),
            &[amount],
        )?;

        Ok(res)
    }

    pub fn execute_transfer_fee(
        &mut self,
        sender: Addr,
        proxy: Addr,
        transfer_fee: Option<Coin>,
    ) -> Result<AppResponse, anyhow::Error> {
        let res = self.app.execute_contract(
            sender,
            proxy,
            &ExecuteMsg::Governance(Action::TransferFee(transfer_fee)),
            &[],
        )?;

        Ok(res)
    }
}

pub fn cw721_base() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw721_base::entry::execute,
        cw721_base::entry::instantiate,
        cw721_base::entry::query,
    );
    Box::new(contract)
}

pub fn cw721_proxy_tester() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw721_proxy_tester::contract::execute,
        cw721_proxy_tester::contract::instantiate,
        cw721_proxy_tester::contract::query,
    );
    Box::new(contract)
}
