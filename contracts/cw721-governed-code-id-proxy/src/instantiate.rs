use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{error::ContractError, msg::InstantiateMsg, state::Cw721GovernedCodeIdProxy};

impl Cw721GovernedCodeIdProxy<'_> {
    pub fn instantiate(
        &self,
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        Cw721GovernedCodeIdProxy::default().governance.instantiate(
            deps.branch(),
            env,
            info,
            cw721_governed_proxy::msg::InstantiateMsg {
                origin: msg.origin.clone(),
                transfer_fee: msg.transfer_fee.clone(),
            },
        )?;
        self.whitelist.init(deps, msg.whitelist.clone())?;
        Ok(Response::default()
            .add_attribute("method", "instantiate")
            .add_attribute("origin", format!("{:?}", msg.origin))
            .add_attribute("transfer_fee", format!("{:?}", msg.transfer_fee))
            .add_attribute("whitelist", format!("{:?}", msg.whitelist)))
    }
}
