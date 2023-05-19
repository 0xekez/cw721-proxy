use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};

use crate::{msg::QueryMsg, state::Cw721GovernanceProxy};

impl Cw721GovernanceProxy<'_> {
    pub fn query(&self, deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::Owner {} => to_binary(&self.load_owner(deps.storage)?),
            QueryMsg::Origin {} => to_binary(&self.load_origin(deps.storage)?),
            QueryMsg::TransferFee {} => to_binary(&self.load_transfer_fee(deps.storage)?),
        }
    }
}
