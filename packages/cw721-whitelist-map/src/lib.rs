use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::{KeyDeserialize, Map, PrimaryKey};
use serde::{de::DeserializeOwned, Serialize};

pub struct WhiteListMap<'a, K, T> {
    pub map: Map<'a, K, T>,
}

impl<'a, K, T> WhiteListMap<'a, K, T>
where
    K: PrimaryKey<'a> + KeyDeserialize,
    T: Serialize + DeserializeOwned,
{
    pub const fn new() -> Self {
        Self {
            map: Map::new("whitelist"),
        }
    }

    pub fn has(&self, storage: &dyn Storage, key: K) -> bool {
        self.map.has(storage, key)
    }

    pub fn load(&self, storage: &dyn Storage, key: K) -> StdResult<T> {
        self.map.load(storage, key)
    }

    pub fn query_is_whitelisted<P>(
        &self,
        storage: &dyn Storage,
        key: K,
        mut predicate: P,
    ) -> StdResult<bool>
    where
        P: FnMut(T) -> bool,
    {
        let value = self.load(storage, key)?;
        let valid = predicate(value);
        Ok(valid)
    }

    pub fn save(&self, storage: &mut dyn Storage, key: K, value: &T) -> StdResult<()> {
        self.map.save(storage, key, value)
    }

    pub fn remove(&self, storage: &mut dyn Storage, key: K) {
        self.map.remove(storage, key);
    }
}
