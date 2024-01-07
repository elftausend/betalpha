mod to_client_packets {
    use super::super::parse::PacketSerializer;
    use super::super::PacketError;
    use crate::packet::parse::{Deserialize, PacketDeserializer, Serialize};
    use betalpha_derive::{Deserialize, Serialize};
    use bytes::Buf;
    use std::io::Cursor;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct KeepAlive;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LoginResponsePacket {
        pub entity_id: i32,
        pub _unused1: String,
        pub _unused2: String,
        pub map_seed: i64,
        pub dimension: u8,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HandshakePacket {
        pub connection_hash: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ChatMessagePacket {
        pub message: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TimeUpdatePacket {
        pub time: u64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PlayerInventoryPacket {
        pub inventory_type: i32,
        pub count: u16,
        pub payload: Vec<u8>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SpawnPositionPacket {
        pub x: i32,
        pub y: i32,
        pub z: i32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UpdateHealthPacket {
        pub health: u8,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RespawnPacket;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PlayerPositionLookPacket {
        pub x: f64,
        pub stance: f64,
        pub y: f64,
        pub z: f64,
        pub yaw: f32,
        pub pitch: f32,
        pub on_ground: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HoldingChangePacket {
        pub entity_id: u32,
        pub item_id: u16,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AddToInventoryPacket {
        pub item_type: u16,
        pub count: u8,
        pub life: u16,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AnimationPacket {
        pub entity_id: u32,
        pub animate: u8,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
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

    #[derive(Debug, Clone, Serialize, Deserialize)]
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

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CollectItemPacket {
        pub collected_entity_id: u32,
        pub collector_entity_id: u32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AddObjectOrVehiclePacket {
        pub entity_id: u32,
        pub object_type: u8,
        pub x: i32,
        pub y: i32,
        pub z: i32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MobSpawnPacket {
        pub entity_id: u32,
        pub mob_type: u8,
        pub x: i32,
        pub y: i32,
        pub z: i32,
        pub yaw: i8,
        pub pitch: i8,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EntityVelocityPacket {
        pub entity_id: u32,
        pub vel_x: i16,
        pub vel_y: i16,
        pub vel_z: i16,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DestroyEntityPacket {
        pub entity_id: u32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EntityPacket {
        pub entity_id: u32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EntityRelativeMovePacket {
        pub entity_id: u32,
        pub x: i8,
        pub y: i8,
        pub z: i8,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EntityLookPacket {
        pub entity_id: u32,
        pub yaw: i8,
        pub pitch: i8,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EntityLookRelativeMovePacket {
        pub entity_id: u32,
        pub x: i8,
        pub y: i8,
        pub z: i8,
        pub yaw: i8,
        pub pitch: i8,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EntityTeleportPacket {
        pub entity_id: u32,
        pub x: i32,
        pub y: i32,
        pub z: i32,
        pub yaw: i8,
        pub pitch: i8,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EntityStatusPacket {
        pub entity_id: u32,
        pub entity_status: u8,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AttachEntityPacket {
        pub entity_id: u32,
        pub vehicle_id: u32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PreChunkPacket {
        pub x: i32,
        pub z: i32,
        pub mode: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
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

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct BlockChangePacket {
        pub x: i32,
        pub y: i8,
        pub z: i32,
        pub block_type: u8,
        pub block_metadata: u8,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ComplexEntitiesPacket {
        pub x: i32,
        pub y: i16,
        pub z: i32,
        pub payload_size: u16,
        pub payload: Vec<u8>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ExplosionPacket {
        pub x: f64,
        pub y: f64,
        pub z: f64,
        pub radius: f32,
        pub record_count: u32,
        pub records: Vec<u8>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct KickPacket {
        pub reason: String,
    }
}

pub mod to_server_packets {
    use super::super::parse::PacketSerializer;
    use super::super::PacketError;
    use betalpha_derive::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct KeepAlivePacket;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LoginRequestPacket {
        pub protocol_version: u32,
        pub username: String,
        pub password: String,
        pub map_seed: u64,
        pub dimension: u8,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HandshakePacket {
        pub connection_hash: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ChatMessagePacket {
        pub message: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PlayerInventoryPacket {
        pub inventory_type: i32,
        pub count: u16,
        pub payload: Vec<u8>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UseEntityPacket {
        pub entity_id: u32,
        pub target_id: u32,
        pub is_left_click: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RespawnPacket;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PlayerPacket {
        pub on_ground: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PlayerPositionPacket {
        pub x: f64,
        pub y: f64,
        pub stance: f64,
        pub z: f64,
        pub on_ground: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PlayerLookPacket {
        pub yaw: f32,
        pub pitch: f32,
        pub on_ground: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PlayerPositionLookPacket {
        pub x: f64,
        pub y: f64,
        pub stance: f64,
        pub z: f64,
        pub yaw: f32,
        pub pitch: f32,
        pub on_ground: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PlayerDiggingPacket {
        pub status: u8,
        pub x: i32,
        pub y: i8,
        pub z: i32,
        pub face: u8,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PlayerBlockPlacementPacket {
        pub item_id: u16,
        pub x: i32,
        pub y: i8,
        pub z: i32,
        pub face: u8,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HoldingChangePacket {
        pub _unused: i32,
        pub item_id: u16,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ArmAnimationPacket {
        pub entity_id: u32,
        pub animate: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
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

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DisconnectPacket {
        pub reason: String,
    }
}

pub use to_client_packets::*;
pub use to_server_packets::*;
