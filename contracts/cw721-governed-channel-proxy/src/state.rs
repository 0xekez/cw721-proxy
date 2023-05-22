use cosmwasm_std::Storage;
use cw721_governed_proxy::state::Cw721GovernanceProxy;
use cw721_whitelist::Whitelist;

use crate::error::ContractError;

pub const CONTRACT_NAME: &str = "crates.io:cw721-governed-channel-proxy";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Cw721GovernedChannelProxy<'a> {
    pub governance: Cw721GovernanceProxy<'a>,
    pub whitelist: Whitelist<'a, String>,
}

impl Default for Cw721GovernedChannelProxy<'_> {
    fn default() -> Self {
        Self {
            governance: Default::default(),
            whitelist: Whitelist::new(),
        }
    }
}

impl Cw721GovernedChannelProxy<'_> {
    pub fn is_whitelisted(
        &self,
        storage: &dyn Storage,
        requestee: String,
    ) -> Result<(), ContractError> {
        match self.whitelist.query_is_whitelisted(storage, &requestee)? {
            true => Ok(()),
            false => Err(ContractError::NotWhitelisted { requestee }),
        }
    }
}
