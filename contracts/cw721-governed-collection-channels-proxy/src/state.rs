use cosmwasm_std::Storage;
use cw721_governed_proxy::state::Cw721GovernanceProxy;
use cw721_whitelist_map::WhiteListMap;

use crate::error::ContractError;

pub const CONTRACT_NAME: &str = "crates.io:cw721-governed-collection-channels-proxy";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Cw721GovernedCollectionChannelsProxy<'a> {
    pub governance: Cw721GovernanceProxy<'a>,
    pub whitelist: WhiteListMap<'a, String, Vec<String>>,
}

impl Default for Cw721GovernedCollectionChannelsProxy<'_> {
    fn default() -> Self {
        Self {
            governance: Default::default(),
            whitelist: WhiteListMap::new(),
        }
    }
}

impl Cw721GovernedCollectionChannelsProxy<'_> {
    pub fn is_whitelisted(
        &self,
        storage: &dyn Storage,
        collection: String,
        channel: String,
    ) -> Result<(), ContractError> {
        if !self.whitelist.has(storage, collection.clone()) {
            Err(ContractError::NotWhitelisted {
                requestee: collection,
            })
        } else {
            match self
                .whitelist
                .query_is_whitelisted(storage, collection, |channels| channels.contains(&channel))?
            {
                true => Ok(()),
                false => Err(ContractError::NotWhitelisted { requestee: channel }),
            }
        }
    }
}
