use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw_rate_limiter::Rate;

use crate::{error::ContractError, msg::InstantiateMsg, state::Cw721GovernedChannelProxy};

impl Cw721GovernedChannelProxy<'_, '_> {
    pub fn instantiate(
        &self,
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        Cw721GovernedChannelProxy::default()
            .governance
            .instantiate(
                deps.branch(),
                env,
                info,
                cw721_governed_proxy::msg::InstantiateMsg {
                    origin: msg.origin.clone(),
                    transfer_fee: msg.transfer_fee.clone(),
                },
            )?;
        if msg.rate_limit.is_zero() {
            Err(ContractError::ZeroRate {})
        } else {
            let (rate, units) = match msg.rate_limit {
                Rate::PerBlock(rate) => (rate, "nfts_per_block"),
                Rate::Blocks(rate) => (rate, "blocks_per_nft"),
            };
            self.rate_limit.init(deps.storage, &msg.rate_limit)?;
            Ok(Response::default()
                .add_attribute("method", "instantiate")
                .add_attribute("origin", format!("{:?}", msg.origin))
                .add_attribute("transfer_fee", format!("{:?}", msg.transfer_fee))
                .add_attribute("rate", rate.to_string())
                .add_attribute("units", units))
        }
    }
}
