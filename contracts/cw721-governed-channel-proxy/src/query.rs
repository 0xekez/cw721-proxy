use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};

use crate::{msg::QueryMsg, state::Cw721GovernedChannelProxy};

impl Cw721GovernedChannelProxy<'_> {
    pub fn query(&self, deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::Owner {} => to_binary(&self.governance.load_owner(deps.storage)?),
            QueryMsg::Origin {} => to_binary(&self.governance.load_origin(deps.storage)?),
            QueryMsg::TransferFee {} => {
                to_binary(&self.governance.load_transfer_fee(deps.storage)?)
            }
            QueryMsg::Whitelist {} => to_binary(&self.whitelist.query_whitelist(deps.storage)?),
            QueryMsg::Whitelisted { value } => {
                to_binary(&self.whitelist.query_is_whitelisted(deps.storage, &value)?)
            }
        }
    }
}
