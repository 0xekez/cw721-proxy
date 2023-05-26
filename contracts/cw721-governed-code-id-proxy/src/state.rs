use cw721_whitelist::Whitelist;

pub const CONTRACT_NAME: &str = "crates.io:cw721-governed_code-id-proxy";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const WHITELIST: Whitelist<u64> = Whitelist::new();
