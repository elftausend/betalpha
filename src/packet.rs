mod types;
mod util;

use std::io::Cursor;
use bytes::Buf;
use crate::packet::ParsingError::InvalidPacketID;
use crate::packet::types::{ToClientPacket, ToServerPacket};
use crate::packet::util::{get_f32, get_f64, get_i32, get_i8, get_inventory_payload, get_string, get_u16, get_u32, get_u64, get_u8};

pub enum ParsingError {
    NotEnoughBytes,
    InvalidString,
    InvalidPacketID(u8),
}

mod ids {
    pub const KEEP_ALIVE: u8 = 0x00;
    pub const LOGIN_REQUEST: u8 = 0x01;
    pub const LOGIN_RESPONSE: u8 = 0x01;
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

pub trait Parseable: Sized {
    fn serialize(self) -> Vec<u8>;
    fn deserialize(bytes: &[u8]) -> Result<Self, ParsingError>;
}

impl Parseable for ToClientPacket {
    fn serialize(self) -> Vec<u8> {
        todo!()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, ParsingError> {
        let mut buf = Cursor::new(&bytes[..]);

        let packet_id = get_u8(&mut buf)?;

        match packet_id {
            _ => { Err(InvalidPacketID(packet_id)) }
        }
    }
}

impl Parseable for ToServerPacket {
    fn serialize(self) -> Vec<u8> {
        todo!()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, ParsingError> {
        let mut buf = Cursor::new(&bytes[..]);

        let packet_id = get_u8(&mut buf)?;

        match packet_id {
            ids::KEEP_ALIVE => Ok(ToServerPacket::KeepAlive),
            ids::LOGIN_REQUEST => Ok(ToServerPacket::LoginRequest {
                protocol_version: get_i32(&mut buf)? as u32,
                username: get_string(&mut buf)?,
                password: get_string(&mut buf)?,
                map_seed: get_u64(&mut buf)?,
                dimension: get_u8(&mut buf)?,
            })
            ,
            ids::HANDSHAKE => Ok(ToServerPacket::Handshake {
                connection_hash: get_string(&mut buf)?,
            }),
            ids::CHAT_MESSAGE => Ok(ToServerPacket::ChatMessage {
                message: get_string(&mut buf)?,
            }),
            ids::PLAYER_INVENTORY => Ok(ToServerPacket::PlayerInventory {
                inventory_type: get_i32(&mut buf)?,
                count: get_u16(&mut buf)?,
                payload: get_inventory_payload(&mut buf)?,
            }),
            ids::USE_ENTITY => Ok(ToServerPacket::UseEntity {
                entity_id: get_u32(&mut buf)?,
                target_id: get_u32(&mut buf)?,
                is_left_click: get_u8(&mut buf)? > 0,
            }),
            ids::RESPAWN => Ok(ToServerPacket::Respawn),
            ids::PLAYER => Ok(ToServerPacket::Player {
                on_ground: get_u8(&mut buf)? > 0,
            }),
            ids::PLAYER_POSITION => Ok(ToServerPacket::PlayerPosition {
                x: get_f64(&mut buf)?,
                y: get_f64(&mut buf)?,
                stance: get_f64(&mut buf)?,
                z: get_f64(&mut buf)?,
                on_ground: get_u8(&mut buf)? > 0,
            }),
            ids::PLAYER_LOOK => Ok(ToServerPacket::PlayerLook {
                yaw: get_f32(&mut buf)?,
                pitch: get_f32(&mut buf)?,
                on_ground: get_u8(&mut buf)? > 0,
            }),
            ids::PLAYER_POSITION_AND_LOOK => Ok(ToServerPacket::PlayerPositionLook {
                x: get_f64(&mut buf)?,
                y: get_f64(&mut buf)?,
                stance: get_f64(&mut buf)?,
                z: get_f64(&mut buf)?,
                yaw: get_f32(&mut buf)?,
                pitch: get_f32(&mut buf)?,
                on_ground: get_u8(&mut buf)? > 0,
            }),
            ids::PLAYER_DIGGING => Ok(ToServerPacket::PlayerDigging {
                status: get_u8(&mut buf)?,
                x: get_i32(&mut buf)?,
                y: get_i8(&mut buf)?,
                z: get_i32(&mut buf)?,
                face: get_u8(&mut buf)?,
            }),
            ids::PLAYER_BLOCK_PLACEMENT => Ok(ToServerPacket::PlayerBlockPlacement {
                item_id: get_u16(&mut buf)?,
                x: get_i32(&mut buf)?,
                y: get_i8(&mut buf)?,
                z: get_i32(&mut buf)?,
                face: get_u8(&mut buf)?,
            }),
            ids::HOLDING_CHANGE => Ok(ToServerPacket::HoldingChange {
                _unused: get_i32(&mut buf)?,
                item_id: get_u16(&mut buf)?,
            }),
            ids::ANIMATION => Ok(ToServerPacket::ArmAnimation {
                entity_id: get_u32(&mut buf)?,
                animate: get_u8(&mut buf)? > 0,
            }),
            ids::PICKUP_SPAWN => Ok(ToServerPacket::PickupSpawn {
                entity_id: get_u32(&mut buf)?,
                item_id: get_u16(&mut buf)?,
                count: get_u8(&mut buf)?,
                x: get_i32(&mut buf)?,
                y: get_i32(&mut buf)?,
                z: get_i32(&mut buf)?,
                rotation: get_i8(&mut buf)?,
                pitch: get_i8(&mut buf)?,
                roll: get_i8(&mut buf)?,
            }),
            ids::KICK_OR_DISCONNECT => Ok(ToServerPacket::Disconnect {
                reason: get_string(&mut buf)?,
            }),
            _ => { Err(InvalidPacketID(packet_id)) }
        }
    }
}