use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg,
};
use cw721::Cw721ReceiveMsg;
use cw721_proxy::ProxyExecuteMsg;

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, MigrateMsg},
    state::Cw721GovernanceProxy,
};

impl<'a> Cw721GovernanceProxy<'a> {
    pub fn execute(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        match msg {
            ExecuteMsg::Owner(addr) => self.execute_owner(deps, env, info, addr),
            ExecuteMsg::Origin(addr) => self.execute_origin(deps, env, info, addr),
            ExecuteMsg::TransferFee(transfer_fee) => {
                self.execute_transfer_fee(deps, env, info, transfer_fee)
            }
            ExecuteMsg::SendFunds { to_address, amount } => {
                self.execute_send_funds(deps, env, info, to_address, amount)
            }
            ExecuteMsg::ReceiveNft(msg) => self.execute_receive_nft(deps, env, info, msg),
            ExecuteMsg::BridgeNft {
                collection,
                token_id,
                msg,
            } => self.execute_bridge_nft(deps, env, info, collection, token_id, msg),
        }
    }
}

impl<'a> Cw721GovernanceProxy<'a> {
    pub fn execute_owner(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        addr: String,
    ) -> Result<Response, ContractError> {
        self.is_owner(deps.storage, info.sender)?;
        let owner = deps.api.addr_validate(&addr)?;
        self.save_owner(deps.storage, &Some(owner))?;
        Ok(Response::default()
            .add_attribute("method", "execute_owner")
            .add_attribute("owner", addr))
    }

    pub fn execute_origin(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        addr: String,
    ) -> Result<Response, ContractError> {
        self.is_owner(deps.storage, info.sender)?;
        let addr = deps.api.addr_validate(&addr)?;
        self.save_origin(deps.storage, &addr)?;
        Ok(Response::default()
            .add_attribute("method", "execute_origin")
            .add_attribute("origin", addr))
    }

    pub fn execute_transfer_fee(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        transfer_fee: Option<Coin>,
    ) -> Result<Response, ContractError> {
        self.is_owner(deps.storage, info.sender)?;
        self.save_transfer_fee(deps.storage, &transfer_fee)?;
        let transfer_fee_string = match transfer_fee {
            Some(fee) => format!("{} {}", fee.amount, fee.denom),
            None => "".to_string(),
        };
        Ok(Response::default()
            .add_attribute("method", "execute_transfer_fee")
            .add_attribute("transfer_fee", transfer_fee_string))
    }

    pub fn execute_send_funds(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        to_address: String,
        amount: Coin,
    ) -> Result<Response, ContractError> {
        self.is_owner(deps.storage, info.sender)?;
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
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        collection: String,
        token_id: String,
        msg: Binary,
    ) -> Result<Response, ContractError> {
        self.check_paid(deps.storage, &info)?;
        Cw721GovernanceProxy::check_approval(
            &deps,
            token_id.clone(),
            collection.clone(),
            env.contract.address.to_string(),
        )?;

        let origin = self.load_origin(deps.storage)?.into_string();
        let transfer_nft_msg = WasmMsg::Execute {
            contract_addr: collection.to_string(), // sender is collection
            msg: to_binary(&cw721::Cw721ExecuteMsg::TransferNft {
                recipient: origin.clone(),
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
            contract_addr: origin, // ICS721
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
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: Cw721ReceiveMsg,
    ) -> Result<Response, ContractError> {
        self.check_paid(deps.storage, &info)?;
        Ok(Response::default().add_message(WasmMsg::Execute {
            contract_addr: self.load_origin(deps.storage)?.into_string(), // ICS721
            msg: to_binary(&ProxyExecuteMsg::ReceiveProxyNft {
                eyeball: info.sender.to_string(),
                msg,
            })?,
            funds: vec![],
        }))
    }

    /// Migrates the contract from the previous version to the current
    /// version.
    pub fn migrate(&self, deps: DepsMut, _env: Env, msg: MigrateMsg) -> StdResult<Response> {
        match msg {
            MigrateMsg::WithUpdate {
                origin,
                transfer_fee,
            } => {
                if let Some(origin) = origin.clone() {
                    let addr = deps.api.addr_validate(&origin)?;
                    self.save_origin(deps.storage, &addr)?;
                }
                self.save_transfer_fee(deps.storage, &transfer_fee)?;
                Ok(Response::default()
                    .add_attribute("method", "migrate")
                    .add_attribute("origin", format!("{:?}", origin))
                    .add_attribute("transfer_fee", format!("{:?}", transfer_fee)))
            }
        }
    }
}
