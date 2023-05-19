use cosmwasm_std::{from_binary, Binary, DepsMut, Env, MessageInfo, Response};
use cw721::Cw721ReceiveMsg;
use cw_rate_limiter::Rate;
use ibc_outgoing_msg::IbcOutgoingMsg;

use crate::{error::ContractError, msg::ExecuteMsg, state::Cw721GovernedChannelProxy};

impl Cw721GovernedChannelProxy<'_, '_> {
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
            ExecuteMsg::RateLimit(rate) => self.execute_rate_limit(deps, env, info, rate),
        }
    }

    pub fn execute_rate_limit(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        rate_limit: Rate,
    ) -> Result<Response, ContractError> {
        self.governance.is_owner(deps.storage, info.sender)?;
        if rate_limit.is_zero() {
            Err(ContractError::ZeroRate {})
        } else {
            self.rate_limit.init(deps.storage, &rate_limit)?;
            let (rate, units) = match rate_limit {
                Rate::PerBlock(rate) => (rate, "nfts_per_block"),
                Rate::Blocks(rate) => (rate, "blocks_per_nft"),
            };
            Ok(Response::default()
                .add_attribute("method", "execute_rate_limit")
                .add_attribute("rate", rate.to_string())
                .add_attribute("units", units))
        }
    }

    pub fn execute_receive_nft(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Cw721ReceiveMsg,
    ) -> Result<Response, ContractError> {
        let IbcOutgoingMsg {
            channel_id: _,
            memo: _,
            receiver: _,
            timeout: _,
        } = from_binary(&msg.msg)?;
        self.rate_limit
            .limit(deps.storage, &env, info.sender.as_str())?;
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
            channel_id: _,
            memo: _,
            receiver: _,
            timeout: _,
        } = from_binary(&msg)?;
        self.rate_limit
            .limit(deps.storage, &env, info.sender.as_str())?;
        Ok(self
            .governance
            .execute_bridge_nft(deps, env, info, collection, token_id, msg)?)
    }
}
