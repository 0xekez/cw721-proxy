use cw_storage_plus::Item;

pub const LAST_MSG: Item<crate::msg::ExecuteMsg> = Item::new("last_msg");
