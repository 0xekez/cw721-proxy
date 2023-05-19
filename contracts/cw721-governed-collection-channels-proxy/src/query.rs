use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdResult, Storage};

use crate::{
    msg::{QueryMsg, SenderToChannelsResponse},
    state::Cw721GovernedCollectionChannelsProxy,
};

impl Cw721GovernedCollectionChannelsProxy<'_> {
    pub fn query(&self, deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::Owner {} => to_binary(&self.governance.load_owner(deps.storage)?),
            QueryMsg::Origin {} => to_binary(&self.governance.load_origin(deps.storage)?),
            QueryMsg::TransferFee {} => {
                to_binary(&self.governance.load_transfer_fee(deps.storage)?)
            }
            QueryMsg::Whitelist {} => to_binary(&self.query_whitelist(deps.storage)?),
            QueryMsg::Whitelisted {
                collection,
                channel,
            } => to_binary(&self.whitelist.query_is_whitelisted(
                deps.storage,
                collection,
                |channels| channels.contains(&channel),
            )?),
        }
    }

    pub fn query_whitelist(
        &self,
        storage: &dyn Storage,
    ) -> StdResult<Vec<SenderToChannelsResponse>> {
        self.whitelist
            .map
            .range(storage, None, None, Order::Ascending)
            .map(|p| {
                let (collection, channels) = p?;
                Ok(SenderToChannelsResponse {
                    collection,
                    channels,
                })
            })
            .collect::<StdResult<Vec<SenderToChannelsResponse>>>()
    }
}
