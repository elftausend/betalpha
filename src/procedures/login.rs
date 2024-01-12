use std::{
    io::Cursor,
    sync::atomic::{AtomicBool, Ordering},
};

use tokio::{
    net::TcpStream,
    sync::{mpsc, RwLock},
};

use crate::{
    get_id,
    packet::{self, util::SendPacket, Deserialize, Item, PacketError},
    world::send_chunk,
    Chunk, PositionAndLook, State,
};

pub async fn login(
    stream: &mut TcpStream,
    buf: &mut Cursor<&[u8]>,
    spawn_chunks: &[Chunk],
    logged_in: &AtomicBool,
    state: &RwLock<State>,
    tx_entity: &mpsc::Sender<(i32, PositionAndLook, Option<String>)>,
) -> Result<(), PacketError> {
    let login_request = packet::LoginRequestPacket::nested_deserialize(buf)?;
    let protocol_version = login_request.protocol_version;
    let username = login_request.username;

    let entity_id = get_id();
    // let seed = 1111423422i64;
    let seed: i64 = 9065250152070435348;
    // let seed: i64 = -4264101711260417039;
    let dimension = 0i8; // -1 hell

    let login_response = packet::LoginResponsePacket {
        entity_id,
        _unused1: String::new(),
        _unused2: String::new(),
        map_seed: seed,
        dimension,
    };
    login_response.send(stream).await?;

    println!("protocol_version {protocol_version}");
    println!("username {username}");
    {
        let mut state = state.write().await;
        state.username = username;
        state.entity_id = entity_id;
    }
    logged_in.store(true, Ordering::Relaxed);

    for chunk in spawn_chunks.iter() {
        send_chunk(chunk, stream).await.unwrap();
    }
    println!("sent map");

    packet::SpawnPositionPacket {
        x: -56i32,
        y: 80i32,
        z: 70i32,
    }
    .send(stream)
    .await?;

    println!("sent spawn");

    for (id, count) in [(-1i32, 36i16), (-2, 4), (-3, 4)] {
        let mut items = vec![None; count as usize];
        if id == -1 {
            items[1] = Some(Item {
                item_id: 51, // 54: chest, 51: fire
                count: 64,
                uses: 0,
            });
        }
        packet::PlayerInventoryPacket {
            inventory_type: id,
            count,
            items,
        }
        .send(stream)
        .await?;
    }

    println!("sent inv");

    let x = 0.27f64;
    let y = 74.62f64;
    let z = 0.65f64;
    let stance: f64 = y + 1.6;

    let yaw = 0f32;
    let pitch = 0f32;

    let outer_state;
    {
        let mut state = state.write().await;
        state.position_and_look.x = x;
        state.position_and_look.y = y;
        state.position_and_look.z = z;
        state.position_and_look.yaw = yaw;
        state.position_and_look.pitch = pitch;
        outer_state = (
            state.entity_id,
            state.position_and_look,
            Some(state.username.clone()),
        );
    }
    tx_entity
        .send((outer_state.0, outer_state.1, outer_state.2))
        .await
        .unwrap();

    let on_ground = true;

    packet::ServerPositionLookPacket {
        x,
        stance,
        y,
        z,
        yaw,
        pitch,
        on_ground,
    }
    .send(stream)
    .await?;

    println!("sent pos");

    state.write().await.logged_in = true;
    Ok(())
}
