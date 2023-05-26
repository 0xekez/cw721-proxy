use cw721_whitelist::Whitelist;

pub const CONTRACT_NAME: &str = "crates.io:cw721-governed-channel-proxy";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const WHITELIST: Whitelist<String> = Whitelist::new();
