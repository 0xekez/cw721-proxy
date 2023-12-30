use cw_paginate_storage::paginate_map_keys;
use cw_storage_plus::{Item, Map};

use cosmwasm_std::{
    Addr, Api, Deps, IbcPacket, MessageInfo, Order, Response, StdError, StdResult, Storage,
};
use ics721_types::ibc_types::NonFungibleTokenPacketData;
use thiserror::Error;

const ORIGIN: Item<Addr> = Item::new("origin");
const SOURCE_CHANNELS: Map<String, String> = Map::new("source_channels");

#[derive(Error, Debug, PartialEq)]
pub enum IncomingProxyError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("Unauthorized source channel: {0}")]
    UnauthorizedSourceChannel(String),

    #[error("Sender is not origin contract: {0}")]
    UnauthorizedOrigin(String),
}

pub trait IncomingProxyExecute {
    fn initialize(
        &self,
        storage: &mut dyn Storage,
        api: &dyn Api,
        origin: Option<String>,
        source_channels: Option<Vec<String>>,
    ) -> StdResult<()> {
        if let Some(origin) = origin {
            ORIGIN.save(storage, &api.addr_validate(&origin)?)?;
        }
        if let Some(source_channels) = source_channels {
            for source_channel in source_channels {
                SOURCE_CHANNELS.save(storage, source_channel.clone(), &source_channel)?;
            }
        }
        Ok(())
    }

    fn execute_ics721_receive_packet_msg<T>(
        &self,
        storage: &mut dyn Storage,
        info: &MessageInfo,
        packet: IbcPacket,
        _data: NonFungibleTokenPacketData,
    ) -> Result<Response<T>, IncomingProxyError> {
        self.assert_origin(storage, info.sender.to_string())?;
        self.assert_packet_data(storage, packet)?;
        Ok(Response::default()
            .add_attribute("method", "execute")
            .add_attribute("action", "ics721_receive_packet_msg"))
    }

    fn assert_origin(
        &self,
        storage: &dyn Storage,
        sender: String,
    ) -> Result<(), IncomingProxyError> {
        if let Some(origin) = ORIGIN.may_load(storage)? {
            if origin == sender {
                return Ok(());
            }
        }
        Err(IncomingProxyError::UnauthorizedOrigin(sender))
    }

    fn assert_packet_data(
        &self,
        storage: &dyn Storage,
        packet: IbcPacket,
    ) -> Result<(), IncomingProxyError> {
        if SOURCE_CHANNELS.has(storage, packet.src.channel_id.clone()) {
            return Ok(());
        }
        Err(IncomingProxyError::UnauthorizedSourceChannel(
            packet.src.channel_id,
        ))
    }
}

pub trait IncomingProxyQuery {
    fn get_source_channels(
        &self,
        deps: Deps,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<Vec<String>> {
        paginate_map_keys(deps, &SOURCE_CHANNELS, start_after, limit, Order::Ascending)
    }

    fn get_origin(&self, storage: &dyn Storage) -> StdResult<Option<Addr>> {
        ORIGIN.may_load(storage)
    }
}

#[cfg(test)]
mod tests;
