use cosmwasm_std::{from_binary, Binary, DepsMut, Env, MessageInfo, Response};
use cw721::Cw721ReceiveMsg;
use ibc_outgoing_msg::IbcOutgoingMsg;

use crate::{error::ContractError, msg::ExecuteMsg, state::Cw721GovernedChannelProxy};

impl Cw721GovernedChannelProxy<'_> {
    pub fn execute(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        match msg {
            ExecuteMsg::Origin(origin) => {
                Ok(self.governance.execute_origin(deps, env, info, origin)?)
            }
            ExecuteMsg::Owner(owner) => {
                Ok(self.governance.execute_owner(deps, env, info, owner)?)
            }
            ExecuteMsg::TransferFee(transfer_fee) => {
                Ok(self
                    .governance
                    .execute_transfer_fee(deps, env, info, transfer_fee)?)
            }
            ExecuteMsg::SendFunds { to_address, amount } => Ok(self
                .governance
                .execute_send_funds(deps, env, info, to_address, amount)?),
            ExecuteMsg::ReceiveNft(msg) => self.execute_receive_nft(deps, env, info, msg),
            ExecuteMsg::BridgeNft {
                collection,
                token_id,
                msg,
            } => self.execute_bridge_nft(deps, env, info, collection, token_id, msg),
            ExecuteMsg::AddToWhitelist { value } => {
                self.execute_add_to_whitelist(deps, env, info, &value)
            }
            ExecuteMsg::RemoveFromWhitelist { value } => {
                self.execute_remove_from_whitelist(deps, env, info, &value)
            }
            ExecuteMsg::ClearWhitelist() => self.execute_clear_whitelist(deps, env, info),
        }
    }

    pub fn execute_add_to_whitelist(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        value: &String,
    ) -> Result<Response, ContractError> {
        self.governance.is_owner(deps.storage, info.sender)?;
        self.whitelist.add(deps.storage, value)?;
        Ok(Response::default()
            .add_attribute("method", "execute_add_to_whitelist")
            .add_attribute("value", value.to_string()))
    }

    pub fn execute_remove_from_whitelist(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        value: &String,
    ) -> Result<Response, ContractError> {
        self.governance.is_owner(deps.storage, info.sender)?;
        self.whitelist.remove(deps.storage, value)?;
        Ok(Response::default()
            .add_attribute("method", "execute_remove_from_whitelist")
            .add_attribute("value", value.to_string()))
    }

    pub fn execute_clear_whitelist(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        self.governance.is_owner(deps.storage, info.sender)?;
        self.whitelist.clear(deps.storage)?;
        Ok(Response::default().add_attribute("method", "execute_clear_whitelist"))
    }

    pub fn execute_receive_nft(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Cw721ReceiveMsg,
    ) -> Result<Response, ContractError> {
        let IbcOutgoingMsg {
            channel_id,
            memo: _,
            receiver: _,
            timeout: _,
        } = from_binary(&msg.msg)?;
        self.is_whitelisted(deps.storage, channel_id)?;
        Ok(self.governance.execute_receive_nft(deps, env, info, msg)?)
    }

    pub fn execute_bridge_nft(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        collection: String,
        token_id: String,
        msg: Binary,
    ) -> Result<Response, ContractError> {
        let IbcOutgoingMsg {
            channel_id,
            memo: _,
            receiver: _,
            timeout: _,
        } = from_binary(&msg)?;
        self.is_whitelisted(deps.storage, channel_id)?;
        Ok(self
            .governance
            .execute_bridge_nft(deps, env, info, collection, token_id, msg)?)
    }
}
