use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdResult};

use crate::{msg::InstantiateMsg, state::Cw721GovernanceProxy};

impl<'a> Cw721GovernanceProxy<'a> {
    pub fn instantiate(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> StdResult<Response> {
        self.save_transfer_fee(deps.storage, &msg.transfer_fee)?;
        let owner = match msg.owner {
            Some(owner) => Some(deps.api.addr_validate(&owner)?),
            None => None,
        };
        self.save_owner(deps.storage, &owner)?;

        let origin = msg
            .origin
            .map(|a| deps.api.addr_validate(&a))
            .transpose()?
            .unwrap_or(info.sender);
        self.save_origin(deps.storage, &origin)?;

        Ok(Response::default()
            .add_attribute("method", "instantiate")
            .add_attribute("origin", origin))
    }
}
