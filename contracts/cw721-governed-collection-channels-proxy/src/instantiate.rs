use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdResult};

use crate::{
    error::ContractError, msg::InstantiateMsg, state::Cw721GovernedCollectionChannelsProxy,
};

impl Cw721GovernedCollectionChannelsProxy<'_> {
    pub fn instantiate(
        &self,
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        Cw721GovernedCollectionChannelsProxy::default()
            .governance
            .instantiate(
                deps.branch(),
                env,
                info,
                cw721_governed_proxy::msg::InstantiateMsg {
                    origin: msg.origin.clone(),
                    owner: msg.owner.clone(),
                    transfer_fee: msg.transfer_fee.clone(),
                },
            )?;
        if let Some(list) = msg.whitelist.clone() {
            list.iter()
                .map(|item| {
                    deps.api.addr_validate(item.0.as_str())?;
                    self.whitelist
                        .save(deps.storage, item.0.to_string(), &item.1)
                })
                .collect::<StdResult<Vec<_>>>()?;
        }
        Ok(Response::default()
            .add_attribute("method", "instantiate")
            .add_attribute("origin", format!("{:?}", msg.origin))
            .add_attribute("owner", format!("{:?}", msg.owner))
            .add_attribute("transfer_fee", format!("{:?}", msg.transfer_fee))
            .add_attribute("whitelist", format!("{:?}", msg.whitelist)))
    }
}
