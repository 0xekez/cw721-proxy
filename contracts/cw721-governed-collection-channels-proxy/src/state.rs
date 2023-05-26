use cw721_whitelist_map::WhiteListMap;

pub const CONTRACT_NAME: &str = "crates.io:cw721-governed-collection-channels-proxy";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const WHITELIST: WhiteListMap<String, Vec<String>> = WhiteListMap::new();
