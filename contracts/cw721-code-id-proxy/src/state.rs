use cosmwasm_std::Addr;
use cw_storage_plus::Item;

use crate::WhiteList;

pub const OWNER: Item<Addr> = Item::new("owner");
pub const ORIGIN: Item<Addr> = Item::new("origin");
pub const WHITELIST: WhiteList<u64> = WhiteList::new();
