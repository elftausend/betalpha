mod types;
mod util;

use std::io::Cursor;
use bytes::Buf;
use crate::packet::ParsingError::InvalidPacketID;
use crate::packet::types::{ToClientPacket, ToServerPacket};
use crate::packet::util::get_u8;

pub enum ParsingError {
    NotEnoughBytes,
    InvalidString,
    InvalidPacketID(u8),
}

mod ids {
    const KEEP_ALIVE: u8 = 0x00;
    const LOGIN_REQUEST: u8 = 0x01;
    const LOGIN_RESPONSE: u8 = 0x01;
    const HANDSHAKE: u8 = 0x02;
    const CHAT_MESSAGE: u8 = 0x03;
    const TIME_UPDATE: u8 = 0x04;
    const PLAYER_INVENTORY: u8 = 0x05;
    const SPAWN_POSITION: u8 = 0x06;
    const USE_ENTITY: u8 = 0x07;
    const UPDATE_HEALTH: u8 = 0x08;
    const RESPAWN: u8 = 0x09;
    const PLAYER: u8 = 0x0A;
    const PLAYER_POSITION: u8 = 0x0B;
    const PLAYER_LOOK: u8 = 0x0C;
    const PLAYER_POSITION_AND_LOOK: u8 = 0x0D;
    const PLAYER_DIGGING: u8 = 0x0E;
    const PLAYER_BLOCK_PLACEMENT: u8 = 0x0F;
    const HOLDING_CHANGE: u8 = 0x10;
    const ADD_TO_INVENTORY: u8 = 0x11;
    const ANIMATION: u8 = 0x12;
    const NAMED_ENTITY_SPAWN: u8 = 0x14;
    const PICKUP_SPAWN: u8 = 0x15;
    const COLLECT_ITEM: u8 = 0x16;
    const ADD_OBJECT_OR_VEHICLE: u8 = 0x17;
    const MOB_SPAWN: u8 = 0x18;
    const ENTITY_VELOCITY: u8 = 0x1C;
    const DESTROY_ENTITY: u8 = 0x1D;
    const ENTITY: u8 = 0x1E;
    const ENTITY_RELATIVE_MOVE: u8 = 0x1F;
    const ENTITY_LOOK: u8 = 0x20;
    const ENTITY_LOOK_AND_RELATIVE_MOVE: u8 = 0x21;
    const ENTITY_TELEPORT: u8 = 0x22;
    const ENTITY_STATUS: u8 = 0x26;
    const PRE_CHUNK: u8 = 0x32;
    const MAP_CHUNK: u8 = 0x33;
    const MULTI_BLOCK_CHANGE: u8 = 0x34;
    const BLOCK_CHANGE: u8 = 0x35;
    const COMPLEX_ENTITY: u8 = 0x3B;
    const EXPLOSION: u8 = 0x3C;
    const KICK_OR_DISCONNECT: u8 = 0xFF;
}

pub trait Parseable: Sized {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(bytes: &[u8]) -> Result<Self, ParsingError>;
}

impl Parseable for ToClientPacket {
    fn serialize(&self) -> Vec<u8> {
        todo!()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, ParsingError> {
        let mut buf = Cursor::new(&bytes[..]);

        let packet_id = get_u8(&mut buf)?;

        match packet_id {
            _ => {Err(InvalidPacketID(packet_id))}
        }
    }
}

impl Parseable for ToServerPacket {
    fn serialize(&self) -> Vec<u8> {
        todo!()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, ParsingError> {
        let mut buf = Cursor::new(&bytes[..]);

        let packet_id = get_u8(&mut buf)?;

        match packet_id {
            _ => {Err(InvalidPacketID(packet_id))}
        }
    }
}