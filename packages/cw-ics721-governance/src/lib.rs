use cw721_base::ExecuteMsg as Cw721ExecuteMsg;
use cw_storage_plus::Item;
use std::marker::PhantomData;

use cosmwasm_std::{
    coin, to_binary, Addr, Binary, Coin, CosmosMsg, DepsMut, Empty, Env, MessageInfo, Response,
    StdError, StdResult, Storage, WasmMsg,
};
use cw721_base::helpers::Cw721Contract;
use cw_utils::{may_pay, PaymentError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("Unauthorized addr {addr}")]
    Unauthorized { addr: String },

    #[error(transparent)]
    Payment(#[from] PaymentError),

    #[error("Incorrect payment amount: {0} != {1}")]
    IncorrectPaymentAmount(Coin, Coin),

    #[error("No approval for {spender} in collection {collection}")]
    MissingApproval { spender: String, collection: String },
}

/// A governed contract may have:
/// - an optional owner,
/// - an origin (ICS721) where msgs are forwarded to, and
/// - an optional transfer fee.
///
/// Owner:
/// - used in is_owner(). For example execute_transfer_fee() allows only owner to change fees.
///
/// Origin:
/// - ...
///
/// Transfer Fee:
/// - ...
pub struct Governance<'a> {
    owner: Item<'a, Option<Addr>>,
    origin: Item<'a, Addr>,
    transfer_fee: Item<'a, Option<Coin>>,
}

impl<'a> Governance<'a> {
    pub const fn new() -> Self {
        Self {
            transfer_fee: Item::new("transfer_fee"),
            owner: Item::new("owner"),
            origin: Item::new("origin"),
        }
    }

    /// Only owner (if given) can change ownership.
    pub fn save_owner(&self, storage: &mut dyn Storage, addr: &Option<Addr>) -> StdResult<()> {
        self.owner.save(storage, addr)
    }

    /// Only owner (if given) can change ownership.
    pub fn save_origin(&self, storage: &mut dyn Storage, addr: &Addr) -> StdResult<()> {
        self.origin.save(storage, addr)
    }

    /// Only owner (if given) can change transfer fee.
    pub fn save_transfer_fee(
        &self,
        storage: &mut dyn Storage,
        transfer_fee: &Option<Coin>,
    ) -> StdResult<()> {
        self.transfer_fee.save(storage, transfer_fee)
    }

    pub fn load_owner(&self, storage: &dyn Storage) -> StdResult<Option<Addr>> {
        match self.owner.may_load(storage).unwrap_or(None) {
            Some(e) => Ok(e),
            None => Ok(None),
        }
    }

    pub fn load_origin(&self, storage: &dyn Storage) -> StdResult<Addr> {
        self.origin.load(storage)
    }

    pub fn load_transfer_fee(&self, storage: &dyn Storage) -> StdResult<Option<Coin>> {
        match self.transfer_fee.may_load(storage).unwrap_or(None) {
            Some(e) => Ok(e),
            None => Ok(None),
        }
    }

    pub fn execute_transfer_fee(
        &self,
        storage: &mut dyn Storage,
        sender: Addr,
        transfer_fee: Option<Coin>,
    ) -> Result<Response, ContractError> {
        self.is_owner(storage, sender)?;
        self.save_transfer_fee(storage, &transfer_fee)?;
        let transfer_fee_string = match transfer_fee {
            Some(fee) => format!("{} {}", fee.amount, fee.denom),
            None => "".to_string(),
        };
        Ok(Response::default()
            .add_attribute("method", "execute_transfer_fee")
            .add_attribute("transfer_fee", transfer_fee_string))
    }

    /// Bridging an NFT requires caller to send funds (if transfer fee is given) and does 2 things:
    /// (1) a sub message to the collection is triggered for sending NFT to this contract.
    /// (2) received submessage forwards msg to ICS721 for interchain transfers.
    pub fn execute_bridge_nft(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        collection: String,
        token_id: String,
        msg: Binary,
    ) -> Result<Response, ContractError> {
        Governance::check_approval(
            &deps,
            token_id.clone(),
            collection.clone(),
            env.contract.address.to_string(),
        )?;
        self.check_paid(deps.storage, &info)?;
        let send_nft_msg: Cw721ExecuteMsg<Empty, Empty> = Cw721ExecuteMsg::SendNft {
            contract: env.contract.address.to_string(), // send nft to this proxy, once send `ReceiveNft` msg in proxy will (1) transfer NFT to ICS721 and (2) forwards ReceiveNft msg to ICS721
            token_id: token_id.clone(),
            msg,
        };
        Ok(Response::default()
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: collection.clone(),
                msg: to_binary(&send_nft_msg)?,
                funds: vec![],
            }))
            .add_attribute("method", "execute_bridge_nft")
            .add_attribute("collection", collection)
            .add_attribute("token_id", token_id))
    }

    /// Throws ContractError::Unauthorized in case sender is not owner (if given).
    pub fn is_owner(&self, storage: &dyn Storage, addr: Addr) -> Result<(), ContractError> {
        match self.load_owner(storage)? {
            Some(owner) => {
                if addr != owner {
                    return Err(ContractError::Unauthorized {
                        addr: addr.to_string(),
                    });
                }
                Ok(())
            }
            None => Ok(()),
        }
    }

    /// Check whether this proxy is eligible to transfer NFT from collection to itself.
    pub fn check_approval(
        deps: &DepsMut,
        token_id: String,
        collection: String,
        spender: String,
    ) -> Result<(), ContractError> {
        let collection_addr = deps.api.addr_validate(&collection)?;
        let approval = Cw721Contract::<Empty, Empty>(collection_addr, PhantomData, PhantomData)
            .approval(&deps.querier, token_id, spender.clone(), None);
        match approval {
            Err(_) => Err(ContractError::MissingApproval {
                spender,
                collection,
            }),
            Ok(_) => Ok(()),
        }
    }

    /// Check whether sender has send funds, based on transfer fee (if given).
    pub fn check_paid(
        &self,
        storage: &dyn Storage,
        info: &MessageInfo,
    ) -> Result<bool, ContractError> {
        match self.load_transfer_fee(storage)? {
            None => Ok(true),
            Some(transaction_fee) => {
                let payment = may_pay(info, &transaction_fee.denom)?;
                if payment != transaction_fee.amount {
                    return Err(ContractError::IncorrectPaymentAmount(
                        coin(payment.u128(), &transaction_fee.denom),
                        transaction_fee,
                    ));
                }
                Ok(true)
            }
        }
    }
}
