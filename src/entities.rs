use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{PositionAndLook, packet::{self, util::SendPacket}, utils::look_to_i8_range};

#[derive(Debug, Clone)]
pub enum Type {
    Player(String),
}

pub async fn spawned_named_entity(
    stream: &mut TcpStream,
    eid: i32,
    name: &str,
    pos_and_look: &PositionAndLook,
) -> Result<(), packet::PacketError> {

    let x = (pos_and_look.x * 32.).round() as i32;
    let y = (pos_and_look.y * 32.).round() as i32;
    let z = (pos_and_look.z * 32.).round() as i32;

    let (rotation, pitch) = look_to_i8_range(pos_and_look.yaw, pos_and_look.pitch);

    packet::NamedEntitySpawnPacket {
        entity_id: eid,
        name: name.to_string(),
        x,
        y,
        z,
        rotation,
        pitch,
        current_item: 0,
    }.send(stream).await
}
