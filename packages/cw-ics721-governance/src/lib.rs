#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

use std::marker::PhantomData;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    coin, to_binary, Addr, Attribute, BankMsg, Binary, Coin, DepsMut, Empty, Env, MessageInfo,
    Response, StdError, StdResult, Storage, WasmMsg,
};
use cw721::Cw721ReceiveMsg;
use cw721_base::helpers::Cw721Contract;
use cw721_proxy::ProxyExecuteMsg;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// re-export the proc macros and the Expiration class
pub use cw_ics721_governance_derive::{cw_ics721_governance_execute, cw_ics721_governance_query};
pub use cw_utils::Expiration;
use cw_utils::{may_pay, PaymentError};

/// Storage constant for the contract's governance
const GOVERNANCE: Item<Governance> = Item::new("governance");

/// A governed contract may have:
/// - an optional owner,
/// - an origin (ICS721) where msgs are forwarded to, and
/// - an optional transfer fee.
///
/// Owner:
/// - used in assert_owner(). For example execute_transfer_fee() allows only owner to change fees.
///
/// Origin:
/// - ...
///
/// Transfer Fee:
/// - ...
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Debug)]
pub struct Governance {
    owner: Option<Addr>,
    origin: Addr,
    transfer_fee: Option<Coin>,
}

impl Governance {
    pub fn into_attributes(self) -> Vec<Attribute> {
        vec![
            Attribute::new(
                "owner",
                self.owner.map_or("none".to_string(), |o| o.to_string()),
            ),
            Attribute::new("origin", self.origin),
            Attribute::new(
                "transfer_fee",
                self.transfer_fee
                    .map_or("none".to_string(), |o| o.to_string()),
            ),
        ]
    }
}

/// Actions that can be taken to alter the proxy contract's governance
#[cw_serde]
pub enum Action {
    /// Changing owner of proxy. Once set, it can't be set to None - except via migration.
    Owner(String),
    /// ICS721 contract where Cw721ReceiveMsg is forwarded to.
    Origin(String),

    /// Optional transfer fee, if provided it will be checked whether funds have been send on `receive_nft` and `bridge_nft` is called.
    /// This means pratically only `bridge_nft` is eligible to call ics721, since `send_nft` is called by collection - and in case of base cw721 it doesn't send funds!
    TransferFee(Option<Coin>),

    /// Send funds from proxy to specified address.
    SendFunds { to_address: String, amount: Coin },

    /// Analogous to `cw721::Cw721ExecuteMsg::SendNft`, where NFT is transferred to ICS721 (escrow) and forwards `Cw721ReceiveMsg` to ICS721.
    BridgeNft {
        collection: String,
        token_id: String,
        msg: Binary,
    },
}

/// Errors associated with the contract's governance
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GovernanceError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Payment(#[from] PaymentError),

    #[error("Contract has no ownership")]
    NoOwner,

    #[error("{0} is not the proxy's current owner")]
    NotOwner(String),

    #[error("Incorrect payment amount: {0} != {1}")]
    IncorrectPaymentAmount(Coin, Coin),

    #[error("{spender} not approved for NFT {token} in collection {collection}")]
    MissingApproval {
        spender: String,
        collection: String,
        token: String,
    },
}

pub fn instantiate(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    origin: Option<String>,
    transfer_fee: Option<Coin>,
) -> StdResult<Response> {
    let owner = match owner {
        Some(owner) => Some(deps.api.addr_validate(owner.as_str())?),
        None => None,
    };
    let origin = origin
        .map(|a| deps.api.addr_validate(&a))
        .transpose()?
        .unwrap_or(info.sender);
    let governance = Governance {
        owner,
        transfer_fee,
        origin,
    };
    GOVERNANCE.save(deps.storage, &governance)?;
    Ok(Response::default()
        .add_attribute("method", "instantiate")
        .add_attributes(governance.into_attributes()))
}

pub fn load(storage: &dyn Storage) -> StdResult<Governance> {
    GOVERNANCE.load(storage)
}

pub fn assert_owner(storage: &dyn Storage, sender: &Addr) -> Result<(), GovernanceError> {
    let governance = load(storage)?;
    match governance.owner {
        Some(owner) => {
            if sender != owner {
                return Err(GovernanceError::NotOwner(sender.to_string()));
            }
            Ok(())
        }
        None => Err(GovernanceError::NoOwner),
    }
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    action: Action,
) -> Result<Response, GovernanceError> {
    match action {
        Action::Owner(owner) => Ok(execute_owner(deps, &info.sender, owner)?),
        Action::Origin(origin) => Ok(execute_origin(deps, &info.sender, origin)?),
        Action::TransferFee(transfer_fee) => {
            Ok(execute_transfer_fee(deps, &info.sender, transfer_fee)?)
        }
        Action::BridgeNft {
            collection,
            token_id,
            msg,
        } => Ok(execute_bridge_nft(
            deps, env, info, collection, token_id, msg,
        )?),
        Action::SendFunds { to_address, amount } => {
            Ok(execute_send_funds(deps, &info.sender, to_address, amount)?)
        }
    }
}

pub fn query_governance(storage: &dyn Storage) -> StdResult<Binary> {
    to_binary(&load(storage)?)
}

pub fn update_owner(
    storage: &mut dyn Storage,
    owner: Option<Addr>,
) -> Result<Governance, GovernanceError> {
    GOVERNANCE.update(storage, |governance| {
        Ok(Governance {
            owner,
            origin: governance.origin,
            transfer_fee: governance.transfer_fee,
        })
    })
}

/// Only owner (if given) can change ownership.
pub fn execute_owner(
    deps: DepsMut,
    sender: &Addr,
    addr: String,
) -> Result<Response, GovernanceError> {
    assert_owner(deps.storage, sender)?;
    let owner = deps.api.addr_validate(&addr)?;
    update_owner(deps.storage, Some(owner))?;
    Ok(Response::default()
        .add_attribute("method", "execute_owner")
        .add_attribute("owner", addr))
}

pub fn update_origin(
    storage: &mut dyn Storage,
    origin: Addr,
) -> Result<Governance, GovernanceError> {
    GOVERNANCE.update(storage, |governance| {
        Ok(Governance {
            owner: governance.owner,
            origin,
            transfer_fee: governance.transfer_fee,
        })
    })
}

/// Only owner (if given) can change origin.
pub fn execute_origin(
    deps: DepsMut,
    sender: &Addr,
    addr: String,
) -> Result<Response, GovernanceError> {
    assert_owner(deps.storage, sender)?;
    let origin = deps.api.addr_validate(&addr)?;
    update_origin(deps.storage, origin)?;
    Ok(Response::default()
        .add_attribute("method", "execute_origin")
        .add_attribute("owner", addr))
}

pub fn update_transfer_fee(
    storage: &mut dyn Storage,
    transfer_fee: Option<Coin>,
) -> Result<Governance, GovernanceError> {
    GOVERNANCE.update(storage, |governance| {
        Ok(Governance {
            owner: governance.owner,
            origin: governance.origin,
            transfer_fee,
        })
    })
}

/// Only owner (if given) can change transfer fee.
pub fn execute_transfer_fee(
    deps: DepsMut,
    sender: &Addr,
    transfer_fee: Option<Coin>,
) -> Result<Response, GovernanceError> {
    assert_owner(deps.storage, sender)?;
    update_transfer_fee(deps.storage, transfer_fee.clone())?;
    Ok(Response::default()
        .add_attribute("method", "execute_transfer_fee")
        .add_attribute(
            "transfer_fee",
            transfer_fee.map_or("".to_string(), |t| format!("{} {}", t.amount, t.denom)),
        ))
}

/// Check whether contract is eligible to transfer NFT from collection to itself.
pub fn check_approval(
    deps: &DepsMut,
    token_id: String,
    collection: String,
    spender: String,
) -> Result<(), GovernanceError> {
    let collection_addr = deps.api.addr_validate(&collection)?;
    let approval = Cw721Contract::<Empty, Empty>(collection_addr, PhantomData, PhantomData)
        .approval(&deps.querier, token_id.clone(), spender.clone(), None);
    match approval {
        Err(_) => Err(GovernanceError::MissingApproval {
            spender,
            collection,
            token: token_id,
        }),
        Ok(_) => Ok(()),
    }
}

pub fn execute_send_funds(
    deps: DepsMut,
    sender: &Addr,
    to_address: String,
    amount: Coin,
) -> Result<Response, GovernanceError> {
    assert_owner(deps.storage, sender)?;
    let msg = BankMsg::Send {
        to_address: to_address.clone(),
        amount: vec![amount.clone()],
    };
    Ok(Response::default()
        .add_message(msg)
        .add_attribute("method", "execute_send_funds")
        .add_attribute("to_address", to_address)
        .add_attribute("amount", format!("{}", amount)))
}

/// Bridging an NFT requires caller to send funds (if transfer fee is given) and does 2 things:
/// (1) a sub message to the collection is triggered for sending NFT to this contract.
/// (2) received submessage forwards msg to ICS721 for interchain transfers.
pub fn execute_bridge_nft(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    collection: String,
    token_id: String,
    msg: Binary,
) -> Result<Response, GovernanceError> {
    check_paid(deps.storage, info)?;
    check_approval(
        &deps,
        token_id.clone(),
        collection.clone(),
        env.contract.address.to_string(),
    )?;

    let governance = load(deps.storage)?;
    let transfer_nft_msg = WasmMsg::Execute {
        contract_addr: collection.to_string(), // sender is collection
        msg: to_binary(&cw721::Cw721ExecuteMsg::TransferNft {
            recipient: governance.origin.to_string(),
            token_id: token_id.clone(),
        })?,
        funds: vec![],
    };

    // forward msg to ICS721, for this sender must be collection
    let receive_msg = Cw721ReceiveMsg {
        msg,
        sender: info.sender.to_string(),
        token_id: token_id.clone(),
    };
    let receive_proxy_msg = WasmMsg::Execute {
        contract_addr: governance.origin.to_string(), // ICS721
        msg: to_binary(&ProxyExecuteMsg::ReceiveProxyNft {
            eyeball: collection.clone(),
            msg: receive_msg,
        })?,
        funds: vec![],
    };

    Ok(Response::default()
        .add_messages(vec![transfer_nft_msg, receive_proxy_msg])
        .add_attribute("method", "execute_bridge_nft")
        .add_attribute("collection", collection)
        .add_attribute("token_id", token_id))
}

/// Delegates receive msg to ICS721 contract.
/// IMPORTANT: in case transfer fee is set, info.funds must contain fee!
/// For example sending funds, using proxy's `BridgeNFT` will work. It won't work using collection's `SendNFT` message!
pub fn execute_receive_nft(
    deps: DepsMut,
    info: MessageInfo,
    msg: Cw721ReceiveMsg,
) -> Result<Response, GovernanceError> {
    check_paid(deps.storage, &info)?;
    Ok(Response::default()
        .add_message(WasmMsg::Execute {
            contract_addr: load(deps.storage)?.origin.into_string(), // ICS721
            msg: to_binary(&ProxyExecuteMsg::ReceiveProxyNft {
                eyeball: info.sender.to_string(),
                msg,
            })?,
            funds: vec![],
        })
        .add_attribute("method", "execute_receive_nft")
        .add_attribute("collection", info.sender))
}

/// Check whether sender has send funds, based on transfer fee (if given).
pub fn check_paid(storage: &dyn Storage, info: &MessageInfo) -> Result<bool, GovernanceError> {
    let governance = GOVERNANCE.load(storage)?;
    match governance.transfer_fee {
        None => Ok(true),
        Some(transaction_fee) => {
            let payment = may_pay(info, &transaction_fee.denom)?;
            if payment != transaction_fee.amount {
                return Err(GovernanceError::IncorrectPaymentAmount(
                    coin(payment.u128(), &transaction_fee.denom),
                    transaction_fee,
                ));
            }
            Ok(true)
        }
    }
}

/// Migrates the contract from the previous version to the current
/// version.
pub fn migrate(
    deps: DepsMut,
    owner: Option<String>,
    origin: Option<String>,
    transfer_fee: Option<Coin>,
) -> StdResult<Response> {
    let mut governance = load(deps.storage)?;
    if let Some(origin) = origin {
        governance.origin = deps.api.addr_validate(&origin)?;
    }
    if let Some(owner) = owner {
        governance.owner = Some(deps.api.addr_validate(&owner)?);
    }
    governance.transfer_fee = transfer_fee;
    GOVERNANCE.save(deps.storage, &governance)?;
    Ok(Response::default()
        .add_attribute("method", "migrate")
        .add_attributes(governance.into_attributes()))
}

#[cfg(test)]
mod tests;
