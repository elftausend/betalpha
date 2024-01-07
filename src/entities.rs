use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::PositionAndLook;

#[derive(Debug, Clone)]
pub enum Type {
    Player(String),
}

pub async fn spawned_named_entity(
    stream: &mut TcpStream,
    eid: i32,
    name: &str,
    pos_and_look: &PositionAndLook,
) {
    let mut named_entity_spawn = vec![0x14];
    named_entity_spawn.extend_from_slice(&eid.to_be_bytes());

    named_entity_spawn.extend_from_slice(&(name.len() as i16).to_be_bytes());
    named_entity_spawn.extend_from_slice(name.as_bytes());
    let x = (pos_and_look.x * 32.).round() as i32;
    let y = (pos_and_look.y * 32.).round() as i32;
    let z = (pos_and_look.z * 32.).round() as i32;

    named_entity_spawn.extend_from_slice(&x.to_be_bytes());
    named_entity_spawn.extend_from_slice(&y.to_be_bytes());
    named_entity_spawn.extend_from_slice(&z.to_be_bytes());
    named_entity_spawn.extend_from_slice(&[0, 0, 0, 0]);

    stream.write_all(&named_entity_spawn).await.unwrap();
    stream.flush().await.unwrap();
}
