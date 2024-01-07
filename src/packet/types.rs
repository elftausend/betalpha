mod to_client_packets {
    use super::super::parse::PacketSerializer;
    use super::super::PacketError;
    use crate::packet::{
        parse::{Deserialize, PacketDeserializer},
        Serialize,
    };
    use betalpha_derive::{serialize, Deserialize};
    use bytes::Buf;
    use std::io::Cursor;
    #[serialize(0)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct KeepAlive;
    #[serialize(0x01)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct LoginResponsePacket {
        pub entity_id: i32,
        pub _unused1: String,
        pub _unused2: String,
        pub map_seed: i64,
        pub dimension: i8,
    }
    #[serialize(0x02)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct HandshakePacket {
        pub connection_hash: String,
    }
    #[serialize(0x03)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct ChatMessagePacket {
        pub message: String,
    }
    #[serialize(0x04)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct TimeUpdatePacket {
        pub time: u64,
    }
    #[serialize(0x05)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct PlayerInventoryPacket {
        pub inventory_type: i32,
        pub count: i16,
        pub payload: Vec<u8>,
    }
    #[serialize(0x06)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct SpawnPositionPacket {
        pub x: i32,
        pub y: i32,
        pub z: i32,
    }
    #[serialize(0x08)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct UpdateHealthPacket {
        pub health: u8,
    }
    #[serialize(0x09)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct RespawnPacket;
    #[serialize(0x0D)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct PlayerPositionLookPacket {
        pub x: f64,
        pub stance: f64,
        pub y: f64,
        pub z: f64,
        pub yaw: f32,
        pub pitch: f32,
        pub on_ground: bool,
    }
    #[serialize(0x10)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct HoldingChangePacket {
        pub entity_id: u32,
        pub item_id: u16,
    }
    #[serialize(0x11)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct AddToInventoryPacket {
        pub item_type: u16,
        pub count: u8,
        pub life: u16,
    }
    #[serialize(0x12)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct AnimationPacket {
        pub entity_id: u32,
        pub animate: u8,
    }
    #[serialize(0x14)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct NamedEntitySpawnPacket {
        pub entity_id: u32,
        pub name: String,
        pub x: i32,
        pub y: i32,
        pub z: i32,
        pub rotation: i8,
        pub pitch: i8,
        pub current_item: u16,
    }
    #[serialize(0x15)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct PickupSpawnPacket {
        pub entity_id: u32,
        pub item_id: u16,
        pub count: u8,
        pub x: i32,
        pub y: i32,
        pub z: i32,
        pub rotation: i8,
        pub pitch: i8,
        pub roll: i8,
    }
    #[serialize(0x16)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct CollectItemPacket {
        pub collected_entity_id: u32,
        pub collector_entity_id: u32,
    }
    #[serialize(0x17)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct AddObjectOrVehiclePacket {
        pub entity_id: u32,
        pub object_type: u8,
        pub x: i32,
        pub y: i32,
        pub z: i32,
    }
    #[serialize(0x18)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct MobSpawnPacket {
        pub entity_id: u32,
        pub mob_type: u8,
        pub x: i32,
        pub y: i32,
        pub z: i32,
        pub yaw: i8,
        pub pitch: i8,
    }
    #[serialize(0x1C)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct EntityVelocityPacket {
        pub entity_id: u32,
        pub vel_x: i16,
        pub vel_y: i16,
        pub vel_z: i16,
    }
    #[serialize(0x1D)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct DestroyEntityPacket {
        pub entity_id: u32,
    }
    #[serialize(0x1E)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct EntityPacket {
        pub entity_id: u32,
    }
    #[serialize(0x1F)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct EntityRelativeMovePacket {
        pub entity_id: u32,
        pub x: i8,
        pub y: i8,
        pub z: i8,
    }
    #[serialize(0x20)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct EntityLookPacket {
        pub entity_id: u32,
        pub yaw: i8,
        pub pitch: i8,
    }
    #[serialize(0x21)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct EntityLookRelativeMovePacket {
        pub entity_id: u32,
        pub x: i8,
        pub y: i8,
        pub z: i8,
        pub yaw: i8,
        pub pitch: i8,
    }
    #[serialize(0x22)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct EntityTeleportPacket {
        pub entity_id: u32,
        pub x: i32,
        pub y: i32,
        pub z: i32,
        pub yaw: i8,
        pub pitch: i8,
    }
    #[serialize(0x26)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct EntityStatusPacket {
        pub entity_id: u32,
        pub entity_status: u8,
    }
    #[serialize(0x27)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct AttachEntityPacket {
        pub entity_id: u32,
        pub vehicle_id: u32,
    }
    #[serialize(0x32)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct PreChunkPacket {
        pub x: i32,
        pub z: i32,
        pub mode: bool,
    }
    #[serialize(0x33)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct MapChunkPacket {
        pub x: i32,
        pub y: i16,
        pub z: i32,
        pub size_x: i8,
        pub size_y: i8,
        pub size_z: i8,
        pub compressed_size: i32,
        pub compressed_data: Vec<u8>,
    }

    #[derive(Debug, Clone)]
    pub struct MultiBlockChangePacket {
        pub chunk_x: i32,
        pub chunk_y: i32,
        pub array_size: u16,
        pub coordinate_array: Vec<i16>,
        pub type_array: Vec<u8>,
        pub metadata_array: Vec<u8>,
    }

    impl Serialize for MultiBlockChangePacket {
        fn serialize(&self) -> Result<Vec<u8>, PacketError> {
            let mut serializer = PacketSerializer::default();
            serializer.serialize_u8(0x34)?;
            serializer.serialize_i32(self.chunk_x)?;
            serializer.serialize_i32(self.chunk_y)?;
            serializer.serialize_u16(self.array_size)?;
            serializer.serialize_payload(
                self.coordinate_array
                    .iter()
                    .map(|e| e.to_be_bytes())
                    .collect::<Vec<[u8; 2]>>()
                    .concat(),
            )?;
            serializer.serialize_payload(self.type_array.clone())?;
            serializer.serialize_payload(self.metadata_array.clone())?;
            Ok(serializer.output)
        }
    }

    impl Deserialize for MultiBlockChangePacket {
        fn nested_deserialize(cursor: &mut Cursor<&[u8]>) -> Result<Self, PacketError> {
            let (_n, chunk_x): (usize, i32) = PacketDeserializer::deserialize_i32(cursor)?;
            let (_n, chunk_y): (usize, i32) = PacketDeserializer::deserialize_i32(cursor)?;
            let (_n, array_size): (usize, u16) = PacketDeserializer::deserialize_u16(cursor)?;
            let remaining = cursor.chunk();
            let coordinate_array = remaining[0..array_size as usize * 2]
                .to_vec()
                .chunks(2)
                .map(|e| i16::from_be_bytes([e[0], e[1]]))
                .collect::<Vec<i16>>();
            let type_array = remaining[array_size as usize * 2..array_size as usize * 3].to_vec();
            let metadata_array =
                remaining[array_size as usize * 3..array_size as usize * 4].to_vec();
            Ok(Self {
                chunk_x,
                chunk_y,
                array_size,
                coordinate_array,
                type_array,
                metadata_array,
            })
        }
    }
    #[serialize(0x35)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct BlockChangePacket {
        pub x: i32,
        pub y: i8,
        pub z: i32,
        pub block_type: u8,
        pub block_metadata: u8,
    }
    #[serialize(0x3B)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct ComplexEntitiesPacket {
        pub x: i32,
        pub y: i16,
        pub z: i32,
        pub payload_size: u16,
        pub payload: Vec<u8>,
    }
    #[serialize(0x3C)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct ExplosionPacket {
        pub x: f64,
        pub y: f64,
        pub z: f64,
        pub radius: f32,
        pub record_count: u32,
        pub records: Vec<u8>,
    }
    #[serialize(0xFF)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct KickPacket {
        pub reason: String,
    }
}

pub mod to_server_packets {
    use super::super::parse::PacketSerializer;
    use crate::PacketError;
    use betalpha_derive::{serialize, Deserialize};

    #[serialize(0x00)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct KeepAlivePacket;
    #[serialize(0x01)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct LoginRequestPacket {
        pub protocol_version: u32,
        pub username: String,
        pub password: String,
        pub map_seed: u64,
        pub dimension: u8,
    }
    #[serialize(0x02)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct HandshakePacket {
        pub connection_hash: String,
    }
    #[serialize(0x03)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct ChatMessagePacket {
        pub message: String,
    }

    #[serialize(0x07)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct UseEntityPacket {
        pub entity_id: u32,
        pub target_id: u32,
        pub is_left_click: bool,
    }
    #[serialize(0x09)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct RespawnPacket;
    #[serialize(0x0A)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct PlayerPacket {
        pub on_ground: bool,
    }
    #[serialize(0x0B)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct PlayerPositionPacket {
        pub x: f64,
        pub y: f64,
        pub stance: f64,
        pub z: f64,
        pub on_ground: bool,
    }
    #[serialize(0x0C)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct PlayerLookPacket {
        pub yaw: f32,
        pub pitch: f32,
        pub on_ground: bool,
    }
    #[serialize(0x0D)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct PlayerPositionLookPacket {
        pub x: f64,
        pub y: f64,
        pub stance: f64,
        pub z: f64,
        pub yaw: f32,
        pub pitch: f32,
        pub on_ground: bool,
    }
    #[serialize(0x0E)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct PlayerDiggingPacket {
        pub status: u8,
        pub x: i32,
        pub y: i8,
        pub z: i32,
        pub face: u8,
    }
    #[serialize(0x0F)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct PlayerBlockPlacementPacket {
        pub item_id: u16,
        pub x: i32,
        pub y: i8,
        pub z: i32,
        pub face: u8,
    }
    #[serialize(0x10)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct HoldingChangePacket {
        pub _unused: i32,
        pub item_id: u16,
    }
    #[serialize(0x12)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct ArmAnimationPacket {
        pub entity_id: u32,
        pub animate: bool,
    }
    #[serialize(0x15)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct PickupSpawnPacket {
        pub entity_id: u32,
        pub item_id: u16,
        pub count: u8,
        pub x: i32,
        pub y: i32,
        pub z: i32,
        pub rotation: i8,
        pub pitch: i8,
        pub roll: i8,
    }
    #[serialize(0xFF)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct DisconnectPacket {
        pub reason: String,
    }
}

pub use to_client_packets::*;
pub use to_server_packets::*;
