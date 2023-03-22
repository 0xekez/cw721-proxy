use cosmwasm_std::{StdResult, Storage, DepsMut};
use cw_storage_plus::{Item};


pub struct WhiteList<'a> {
    whitelist: Item<'a, Vec<String>>,
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
            .collect();
        self.whitelist.save(deps.storage, &whitelist)
    }

    pub fn query_whitelist(&self, storage: &dyn Storage) -> StdResult<Vec<String>> {
        self.whitelist.load(storage)
    }

    pub fn query_is_whitelisted(&self, storage: &dyn Storage, value: &String) -> StdResult<bool> {
        let whitelist = self.query_whitelist(storage)?;
        Ok(whitelist.contains(value))
    }

    pub fn add(&self, storage: &mut dyn Storage, value: &String) -> StdResult<()> {
        let mut whitelist = self.query_whitelist(storage)?;
        match whitelist.contains(value) {
            true => Ok(()),
            false => {
                whitelist.push(value.clone());
                self.whitelist.save(storage, &whitelist)?;
                Ok(())
            }
        }
    }

    pub fn remove(&self, storage: &mut dyn Storage, value: &String) -> StdResult<()> {
        let mut whitelist = self.query_whitelist(storage)?;
        match whitelist.contains(value) {
            true => {
                whitelist.remove(whitelist.iter().position(|x| x == value).unwrap());
                self.whitelist.save(storage, &whitelist)?;
                Ok(())
            }
            false => Ok(()),
        }
    }
}