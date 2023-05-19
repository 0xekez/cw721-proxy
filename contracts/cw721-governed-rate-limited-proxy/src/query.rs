use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};

use crate::{msg::QueryMsg, state::Cw721GovernedChannelProxy};

impl Cw721GovernedChannelProxy<'_, '_> {
    pub fn query(&self, deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::Owner {} => to_binary(&self.governance.load_owner(deps.storage)?),
            QueryMsg::Origin {} => to_binary(&self.governance.load_origin(deps.storage)?),
            QueryMsg::TransferFee {} => {
                to_binary(&self.governance.load_transfer_fee(deps.storage)?)
            }
            QueryMsg::RateLimit {} => to_binary(&self.rate_limit.query_limit(deps.storage)?),
        }
    }
}
