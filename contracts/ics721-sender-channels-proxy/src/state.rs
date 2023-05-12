use cw721_whitelist_map::WhiteListMap;
use cw_ics721_governance::Governance;

pub const GOVERNANCE: Governance = Governance::new();
pub const SENDER_TO_CHANNELS: WhiteListMap<String, Vec<String>> = WhiteListMap::new();
