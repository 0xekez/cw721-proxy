use std::marker::PhantomData;

use cosmwasm_std::{coin, Addr, Coin, DepsMut, Empty, MessageInfo, StdResult, Storage};
use cw721_base::helpers::Cw721Contract;
use cw_storage_plus::Item;
use cw_utils::may_pay;

use crate::error::ContractError;

// Version info for migration
pub const CONTRACT_NAME: &str = "crates.io:cw721-governed-proxy";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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
pub struct Cw721GovernanceProxy<'a> {
    owner: Item<'a, Option<Addr>>,
    origin: Item<'a, Addr>,
    transfer_fee: Item<'a, Option<Coin>>,
}

impl Default for Cw721GovernanceProxy<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Cw721GovernanceProxy<'a> {
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
            None => Err(ContractError::Unauthorized {
                addr: addr.to_string(),
            }),
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
