use cosmwasm_std::Addr;
use cw721_whitelist_map::WhiteListMap;
use cw_storage_plus::Item;

pub const OWNER: Item<Addr> = Item::new("owner");
pub const ORIGIN: Item<Addr> = Item::new("origin");
pub const SENDER_TO_CHANNELS: WhiteListMap<String, Vec<String>> = WhiteListMap::new();
