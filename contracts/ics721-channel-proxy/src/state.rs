use crate::WhiteList;
use cw_ics721_governance::Governance;

pub const GOVERNANCE: Governance = Governance::new();
pub const WHITELIST: WhiteList<String> = WhiteList::new();
