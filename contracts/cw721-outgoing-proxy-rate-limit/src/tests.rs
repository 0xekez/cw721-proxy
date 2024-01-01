use cosmwasm_std::{from_json, to_json_binary, Addr, Empty, IbcTimeout};
use cw721::Cw721ReceiveMsg;
use cw_multi_test::{next_block, App, Contract, ContractWrapper, Executor};
use cw_rate_limiter::{Rate, RateLimitError};
use ics721_types::ibc_types::{IbcOutgoingMsg, IbcOutgoingProxyMsg};

use crate::msg::{InstantiateMsg, QueryMsg};

struct Test {
    pub app: App,
    pub cw721s: Vec<Addr>,
    pub minter: Addr,
    pub rate_limiter: Addr,
    pub mock_receiver: Addr,

    nfts_minted: usize,
    rate_limiter_id: u64,
}

impl Test {
    pub fn new(cw721s: usize, rate: Rate) -> Self {
        let mut app = App::default();
        let minter = Addr::unchecked("minter");

        let cw721_id = app.store_code(cw721_base());
        let rate_limiter_id = app.store_code(cw721_rate_limiter());
        let proxy_tester_id = app.store_code(cw721_proxy_tester());

        let mock_receiver = app
            .instantiate_contract(
                proxy_tester_id,
                minter.clone(),
                &cw721_outgoing_proxy_tester::msg::InstantiateMsg::default(),
                &[],
                "proxy_tester",
                None,
            )
            .unwrap();

        let rate_limiter = app
            .instantiate_contract(
                rate_limiter_id,
                minter.clone(),
                &InstantiateMsg::new(rate, Some(mock_receiver.to_string())),
                &[],
                "rate_limiter",
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
            rate_limiter,
            mock_receiver,
            nfts_minted: 0,
            rate_limiter_id,
        }
    }

    pub fn update_rate(&mut self, rate: Rate) -> Result<(), anyhow::Error> {
        self.app
            .instantiate_contract(
                self.rate_limiter_id,
                self.minter.clone(),
                &InstantiateMsg::new(rate, Some(self.mock_receiver.to_string())),
                &[],
                "rate_limiter",
                None,
            )
            .map(|rate_limiter| self.rate_limiter = rate_limiter)
    }

    pub fn send_nft_and_check_received(&mut self, nft: Addr) -> Result<(), anyhow::Error> {
        self.nfts_minted += 1;

        self.app.execute_contract(
            self.minter.clone(),
            nft.clone(),
            &cw721_base::msg::ExecuteMsg::<Empty, Empty>::Mint {
                token_id: self.nfts_minted.to_string(),
                owner: self.minter.to_string(),
                token_uri: None,
                extension: Default::default(),
            },
            &[],
        )?;

        let ibc_msg = IbcOutgoingMsg {
            receiver: "receiver".to_string(),
            channel_id: "channel".to_string(),
            timeout: IbcTimeout::with_timestamp(self.app.block_info().time.plus_minutes(30)),
            memo: None,
        };
        self.app.execute_contract(
            self.minter.clone(),
            nft.clone(),
            &cw721_base::msg::ExecuteMsg::<Empty, Empty>::SendNft {
                contract: self.rate_limiter.to_string(),
                token_id: self.nfts_minted.to_string(),
                msg: to_json_binary(&ibc_msg)?,
            },
            &[],
        )?;

        let msg: cw721_outgoing_proxy_tester::msg::ExecuteMsg = self.app.wrap().query_wasm_smart(
            &self.mock_receiver,
            &cw721_outgoing_proxy_tester::msg::QueryMsg::LastMsg {},
        )?;

        match msg {
            cw721_outgoing_proxy_tester::msg::ExecuteMsg::ReceiveNft(msg) => {
                let Cw721ReceiveMsg {
                    sender,
                    token_id,
                    msg,
                } = msg;
                let msg: IbcOutgoingProxyMsg = from_json(msg)?;
                assert_eq!(sender, self.minter.to_string());
                assert_eq!(token_id, self.nfts_minted.to_string());
                assert_eq!(
                    msg,
                    IbcOutgoingProxyMsg {
                        collection: nft.to_string(),
                        msg: to_json_binary(&ibc_msg)?
                    }
                )
            }
        }

        Ok(())
    }

    pub fn send_nfts_at_rate(
        &mut self,
        cw721: Addr,
        rate: Rate,
        for_blocks: usize,
    ) -> Result<(), anyhow::Error> {
        let start_block = self.app.block_info().height;
        for _ in 0..for_blocks {
            match rate {
                Rate::PerBlock(n) => {
                    for _ in 0..n {
                        self.send_nft_and_check_received(cw721.clone())?;
                    }
                }
                Rate::Blocks(b) => {
                    if (self.app.block_info().height - start_block) % b == 0 {
                        self.send_nft_and_check_received(cw721.clone())?;
                    }
                }
            }
            self.app.update_block(next_block)
        }

        Ok(())
    }
}

impl InstantiateMsg {
    fn new(rate_limit: Rate, origin: Option<String>) -> Self {
        Self { rate_limit, origin }
    }
}

fn cw721_rate_limiter() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_reply(crate::contract::reply);
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
        cw721_outgoing_proxy_tester::contract::execute,
        cw721_outgoing_proxy_tester::contract::instantiate,
        cw721_outgoing_proxy_tester::contract::query,
    );
    Box::new(contract)
}

// Generates a random rate with an internal value within RANGE.
fn random_rate<R: rand::Rng, S: rand::distributions::uniform::SampleRange<u64>>(
    rng: &mut R,
    range: S,
) -> Rate {
    let t = rng.gen();
    let v = rng.gen_range(range) + 1u64;
    match t {
        true => Rate::Blocks(v),
        false => Rate::PerBlock(v + 1),
    }
}

#[test]
fn simple_send() {
    let mut test = Test::new(1, Rate::Blocks(1));
    test.send_nft_and_check_received(test.cw721s[0].clone())
        .unwrap()
}

#[test]
fn test_simple_not_limited() {
    let expected = Rate::PerBlock(2);
    let actual = Rate::Blocks(1);
    let mut test = Test::new(1, expected);
    test.send_nfts_at_rate(test.cw721s[0].clone(), actual, 1)
        .unwrap();
}

#[test]
fn test_simple_rate_limited() {
    let actual = Rate::PerBlock(2);
    let expected = Rate::Blocks(4);
    let mut test = Test::new(1, expected);
    let err: RateLimitError = test
        .send_nfts_at_rate(test.cw721s[0].clone(), actual, 1)
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        RateLimitError::Limited {
            blocks_remaining: 4,
            key: test.cw721s[0].to_string(),
        }
    )
}

#[test]
fn test_multikey_rate_limit() {
    let rate_limit = Rate::PerBlock(2);
    let mut test = Test::new(2, rate_limit);

    test.send_nft_and_check_received(test.cw721s[0].clone())
        .unwrap();
    test.send_nft_and_check_received(test.cw721s[0].clone())
        .unwrap();
    test.send_nft_and_check_received(test.cw721s[1].clone())
        .unwrap();
    let err: RateLimitError = test
        .send_nft_and_check_received(test.cw721s[0].clone())
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        RateLimitError::Limited {
            key: test.cw721s[0].to_string(),
            blocks_remaining: 1
        }
    );
    test.send_nft_and_check_received(test.cw721s[1].clone())
        .unwrap();
    let err: RateLimitError = test
        .send_nft_and_check_received(test.cw721s[1].clone())
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        RateLimitError::Limited {
            key: test.cw721s[1].to_string(),
            blocks_remaining: 1
        }
    );

    test.app.update_block(next_block);

    test.send_nfts_at_rate(test.cw721s[0].clone(), rate_limit, 1)
        .unwrap();
    test.send_nfts_at_rate(test.cw721s[1].clone(), rate_limit, 1)
        .unwrap();
}

#[test]
fn fuzz_rate_limiting() {
    let iterations = 200;
    let max = 15;
    let range = 1..max;
    let rng = &mut rand::thread_rng();

    let limit = random_rate(rng, range.clone());
    let mut test = Test::new(1, limit);

    for _ in 0..iterations {
        let actual = random_rate(rng, range.clone());
        let limit = random_rate(rng, range.clone());
        test.update_rate(limit).unwrap();

        let res = test.send_nfts_at_rate(test.cw721s[0].clone(), actual, max as usize);
        let pass = match actual > limit {
            true => res.is_err(),
            false => res.is_ok(),
        };
        if !pass {
            panic!(
                "test failed on (limit, actual) = ({:?}, {:?})",
                limit, actual
            )
        }

        // Open state for next iteration.
        if let Rate::Blocks(blocks) = limit {
            test.app.update_block(|b| b.height += blocks);
        } else {
            test.app.update_block(next_block)
        }
    }
}

#[test]
fn test_origin_instantiator() {
    let mut app = App::default();
    let rate_limiter_id = app.store_code(cw721_rate_limiter());

    // Check that origin is set to instantiator if origin is None
    // during instantiation.
    let rate_limiter = app
        .instantiate_contract(
            rate_limiter_id,
            Addr::unchecked("ekez"),
            &InstantiateMsg {
                rate_limit: Rate::Blocks(20),
                origin: None,
            },
            &[],
            "rate limiter",
            None,
        )
        .unwrap();

    let origin: Addr = app
        .wrap()
        .query_wasm_smart(&rate_limiter, &QueryMsg::Origin {})
        .unwrap();
    assert_eq!(origin, Addr::unchecked("ekez"));

    let rate: Rate = app
        .wrap()
        .query_wasm_smart(&rate_limiter, &QueryMsg::RateLimit {})
        .unwrap();
    assert_eq!(rate, Rate::Blocks(20))
}

#[test]
fn test_origin_specified() {
    let mut app = App::default();
    let rate_limiter_id = app.store_code(cw721_rate_limiter());

    // Check that origin is set to instantiator if origin is None
    // during instantiation.
    let rate_limiter = app
        .instantiate_contract(
            rate_limiter_id,
            Addr::unchecked("zeke"),
            &InstantiateMsg {
                rate_limit: Rate::PerBlock(20),
                origin: Some("ekez".to_string()),
            },
            &[],
            "rate limiter",
            None,
        )
        .unwrap();

    let origin: Addr = app
        .wrap()
        .query_wasm_smart(&rate_limiter, &QueryMsg::Origin {})
        .unwrap();
    assert_eq!(origin, Addr::unchecked("ekez"));

    let rate: Rate = app
        .wrap()
        .query_wasm_smart(&rate_limiter, &QueryMsg::RateLimit {})
        .unwrap();
    assert_eq!(rate, Rate::PerBlock(20))
}

#[test]
fn test_zero_rate_instantiate() {
    let mut app = App::default();
    let rate_limiter_id = app.store_code(cw721_rate_limiter());

    let err: RateLimitError = app
        .instantiate_contract(
            rate_limiter_id,
            Addr::unchecked("zeke"),
            &InstantiateMsg {
                rate_limit: Rate::PerBlock(0),
                origin: Some("ekez".to_string()),
            },
            &[],
            "rate limiter",
            None,
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, RateLimitError::ZeroRate {});

    let infinity = Rate::Blocks(0);
    assert!(infinity.is_infinite());

    app.instantiate_contract(
        rate_limiter_id,
        Addr::unchecked("zeke"),
        &InstantiateMsg {
            rate_limit: infinity,
            origin: Some("ekez".to_string()),
        },
        &[],
        "rate limiter",
        None,
    )
    .unwrap();
}
