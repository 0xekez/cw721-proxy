use cosmwasm_std::{StdResult, Storage, Addr, DepsMut};
use cw_storage_plus::{Item};


pub struct WhiteList<'a> {
    whitelist: Item<'a, Vec<Addr>>,
}

impl<'a> WhiteList<'a> {
    pub const fn new() -> Self {
        Self {
            whitelist: Item::new("whitelist"),
        }
    }

    pub fn init(&self, deps: DepsMut, whitelist: Option<Vec<String>>) -> StdResult<()> {
        let whitelist = whitelist
            .map_or(vec![], |wl| wl)
            .into_iter()
            .map(|addr| deps.api.addr_validate(&addr).unwrap())
            .collect();
        self.whitelist.save(deps.storage, &whitelist)
    }

    pub fn query_whitelist(&self, storage: &dyn Storage) -> StdResult<Vec<Addr>> {
        self.whitelist.load(storage)
    }

    pub fn query_is_whitelisted(&self, storage: &dyn Storage, addr: &Addr) -> StdResult<bool> {
        let whitelist = self.query_whitelist(storage)?;
        Ok(whitelist.contains(addr))
    }

    pub fn add(&self, storage: &mut dyn Storage, addr: &Addr) -> StdResult<()> {
        let mut whitelist = self.query_whitelist(storage)?;
        match whitelist.contains(addr) {
            true => Ok(()),
            false => {
                whitelist.push(addr.clone());
                self.whitelist.save(storage, &whitelist)?;
                Ok(())
            }
        }
    }

    pub fn remove(&self, storage: &mut dyn Storage, addr: &Addr) -> StdResult<()> {
        let mut whitelist = self.query_whitelist(storage)?;
        match whitelist.contains(addr) {
            true => {
                whitelist.remove(whitelist.iter().position(|x| x == addr).unwrap());
                self.whitelist.save(storage, &whitelist)?;
                Ok(())
            }
            false => Ok(()),
        }
    }
}