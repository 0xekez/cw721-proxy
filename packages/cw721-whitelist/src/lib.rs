use cosmwasm_std::{StdResult, Storage, DepsMut};
use cw_storage_plus::{Item};
use serde::{Serialize, de::DeserializeOwned};


pub struct WhiteList<'a, T> {
    whitelist: Item<'a, Vec<T>>,
}

impl<'a, T> WhiteList<'a, T>
where
    T: Serialize + DeserializeOwned + PartialEq + Clone,
{
    pub const fn new() -> Self {
        Self {
            whitelist: Item::new("whitelist"),
        }
    }

    pub fn init(&self, deps: DepsMut, whitelist: Vec<T>) -> StdResult<()> {
        self.whitelist.save(deps.storage, &whitelist)
    }

    pub fn query_whitelist(&self, storage: &dyn Storage) -> StdResult<Vec<T>> {
        self.whitelist.load(storage)
    }

    pub fn query_is_whitelisted(&self, storage: &dyn Storage, value: &T) -> StdResult<bool> {
        let whitelist = self.query_whitelist(storage)?;
        Ok(whitelist.contains(value))
    }

    pub fn add(&self, storage: &mut dyn Storage, value: &T) -> StdResult<()> {
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

    pub fn remove(&self, storage: &mut dyn Storage, value: &T) -> StdResult<()> {
        let mut whitelist = self.query_whitelist(storage)?;
        match whitelist.contains(value) {
            true => {
                whitelist.remove(whitelist.iter().position(|x| x.eq(value)).unwrap());
                self.whitelist.save(storage, &whitelist)?;
                Ok(())
            }
            false => Ok(()),
        }
    }
}