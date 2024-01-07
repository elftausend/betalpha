mod parse;
mod types;
pub mod util;

pub use parse::*;
pub use types::*;

use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub enum PacketError {
    NotEnoughBytes,
    InvalidString,
    InvalidPacketID(u8),
    InvalidOptional,
    IOError(std::io::Error),
}

impl Error for PacketError {}

impl Debug for PacketError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for PacketError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            PacketError::NotEnoughBytes => "Packet is incomplete".to_string(),
            PacketError::InvalidString => "Provided string is invalid".to_string(),
            PacketError::InvalidPacketID(id) => format!("Packet ID {id} is an invalid packet"),
            PacketError::InvalidOptional => "Invalid optional".to_string(),
            PacketError::IOError(e) => format!("IO error occured: {e}"),
        };
        write!(f, "{text}")
    }
}

mod ids {
    pub const KEEP_ALIVE: u8 = 0x00;
    pub const LOGIN: u8 = 0x01;
    pub const HANDSHAKE: u8 = 0x02;
    pub const CHAT_MESSAGE: u8 = 0x03;
    pub const TIME_UPDATE: u8 = 0x04;
    pub const PLAYER_INVENTORY: u8 = 0x05;
    pub const SPAWN_POSITION: u8 = 0x06;
    pub const USE_ENTITY: u8 = 0x07;
    pub const UPDATE_HEALTH: u8 = 0x08;
    pub const RESPAWN: u8 = 0x09;
    pub const PLAYER: u8 = 0x0A;
    pub const PLAYER_POSITION: u8 = 0x0B;
    pub const PLAYER_LOOK: u8 = 0x0C;
    pub const PLAYER_POSITION_AND_LOOK: u8 = 0x0D;
    pub const PLAYER_DIGGING: u8 = 0x0E;
    pub const PLAYER_BLOCK_PLACEMENT: u8 = 0x0F;
    pub const HOLDING_CHANGE: u8 = 0x10;
    pub const ADD_TO_INVENTORY: u8 = 0x11;
    pub const ANIMATION: u8 = 0x12;
    pub const NAMED_ENTITY_SPAWN: u8 = 0x14;
    pub const PICKUP_SPAWN: u8 = 0x15;
    pub const COLLECT_ITEM: u8 = 0x16;
    pub const ADD_OBJECT_OR_VEHICLE: u8 = 0x17;
    pub const MOB_SPAWN: u8 = 0x18;
    pub const ENTITY_VELOCITY: u8 = 0x1C;
    pub const DESTROY_ENTITY: u8 = 0x1D;
    pub const ENTITY: u8 = 0x1E;
    pub const ENTITY_RELATIVE_MOVE: u8 = 0x1F;
    pub const ENTITY_LOOK: u8 = 0x20;
    pub const ENTITY_LOOK_AND_RELATIVE_MOVE: u8 = 0x21;
    pub const ENTITY_TELEPORT: u8 = 0x22;
    pub const ENTITY_STATUS: u8 = 0x26;
    pub const PRE_CHUNK: u8 = 0x32;
    pub const MAP_CHUNK: u8 = 0x33;
    pub const MULTI_BLOCK_CHANGE: u8 = 0x34;
    pub const BLOCK_CHANGE: u8 = 0x35;
    pub const COMPLEX_ENTITY: u8 = 0x3B;
    pub const EXPLOSION: u8 = 0x3C;
    pub const KICK_OR_DISCONNECT: u8 = 0xFF;
}
