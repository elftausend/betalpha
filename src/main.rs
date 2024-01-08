use std::{
    collections::HashSet,
    future::Future,
    io::Cursor,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        Arc,
    },
};

use bytes::{Buf, BytesMut};

use nbt::{Blob, Value};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{
        broadcast::{self, error::TryRecvError},
        mpsc::{self, Sender},
        RwLock,
    },
};

mod global_handlers;
mod packet;

use crate::packet::PacketError;
use crate::packet::{util::*, Deserialize, Serialize};

// mod byte_man;
// pub use byte_man::*;

mod entities;

fn base36_to_base10(input: i8) -> i32 {
    let mut result = 0;
    let mut base = 1;
    let mut num = input.abs() as i32;

    while num > 0 {
        let digit = num % 10;
        result += digit * base;
        num /= 10;
        base *= 36;
    }

    result * if input.is_negative() { -1 } else { 1 }
}

#[test]
fn test_base_conv() {
    assert_eq!(base36_to_base10(18), 44);
    println!("base: {}", base36_to_base10(-127));
    println!("{}", (12 << 4));
}

use crate::entities::spawned_named_entity;

pub struct Chunk {
    chunk_x: i32,
    chunk_z: i32,
    blocks: Vec<u8>,
    data: Vec<u8>,
    sky_light: Vec<u8>,
    block_light: Vec<u8>,
    height_map: Vec<u8>,
}

type PacketHandler = Box<
    dyn FnOnce(
        &mut Cursor<&[u8]>,
        &mut TcpStream,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>>>>,
>;

#[inline]
pub async fn incomplete(_buf: &mut Cursor<&[u8]>, _stream: &mut TcpStream) -> Result<(), Error> {
    Err(Error::Incomplete)
}

fn force_boxed<T>(f: fn(&mut Cursor<&[u8]>, &mut TcpStream) -> T) -> PacketHandler
where
    T: Future<Output = Result<(), Error>> + 'static,
{
    Box::new(move |buf, stream| Box::pin(f(buf, stream)))
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:25565").await.unwrap();

    // force_boxed::<_>(keep_alive);
    // let mut packet_handlers: Vec<PacketHandler> = vec![force_boxed(incomplete)];
    // packet_handlers[0x00] = keep_alive;

    // let mut chunks = Vec::new();
    let dirs = walkdir::WalkDir::new("./World2/")
        // let dirs = walkdir::WalkDir::new("/home/elftausend/.minecraft/saves/World1/")
        .into_iter()
        .collect::<Vec<_>>();

    // 1. use rayon, 2. try valence_nbt for maybe improved performance?
    let chunks = dirs
        .par_iter()
        .filter_map(|entry| {
            let entry = entry.as_ref().unwrap();
            let useable_filename = entry.path().file_name().unwrap().to_str().unwrap(); // thx rust
            if !useable_filename.ends_with(".dat") || useable_filename.ends_with("level.dat") {
                return None;
            };
            println!("entry path: {:?}", entry.path());
            let mut file = std::fs::File::open(entry.path()).unwrap();

            let blob: Blob = nbt::from_gzip_reader(&mut file).unwrap();

            let Some(Value::Compound(level)) = &blob.get("Level") else {
                println!("INFO: invalid path: {entry:?}");
                return None;
            };

            let x = match level.get("xPos").unwrap() {
                Value::Byte(x) => *x,
                d => panic!("invalid dtype: {d:?}"),
            };

            let chunk_x = base36_to_base10(x);

            let z = match level.get("zPos").unwrap() {
                Value::Byte(x) => *x,
                d => panic!("invalid dtype {d:?}"),
            };

            let chunk_z = base36_to_base10(z);

            let Value::ByteArray(blocks) = &level["Blocks"] else {
                panic!("invalid");
            };
            let blocks = blocks
                .iter()
                .map(|x| x.to_be_bytes()[0])
                .collect::<Vec<_>>();

            let Value::ByteArray(data) = &level["Data"] else {
                panic!("invalid");
            };

            let data = data.iter().map(|x| x.to_be_bytes()[0]).collect::<Vec<_>>();

            let Value::ByteArray(sky_light) = &level["SkyLight"] else {
                panic!("invalid");
            };
            let sky_light = sky_light
                .iter()
                .map(|x| x.to_be_bytes()[0])
                .collect::<Vec<_>>();

            let Value::ByteArray(block_light) = &level["BlockLight"] else {
                panic!("invalid");
            };
            let block_light = block_light
                .iter()
                .map(|x| x.to_be_bytes()[0])
                .collect::<Vec<_>>();

            let Value::ByteArray(height_map) = &level["HeightMap"] else {
                panic!("invalid");
            };
            let height_map = height_map
                .iter()
                .map(|x| x.to_be_bytes()[0])
                .collect::<Vec<_>>();

            Some(Chunk {
                chunk_x,
                chunk_z,
                blocks,
                data,
                sky_light,
                block_light,
                height_map,
            })
        })
        .collect::<Vec<_>>();
    let chunks = &*Box::leak(chunks.into_boxed_slice());
    let (pos_and_look_tx, mut pos_and_look_rx) =
        mpsc::channel::<(i32, PositionAndLook, Option<String>)>(256);

    let (pos_and_look_update_tx, _pos_and_look_update_rx) = broadcast::channel(256);
    let (tx_destroy_self_entity, mut rx_entity_destroy) = mpsc::channel::<i32>(100);
    let (tx_destroy_entities, _) = broadcast::channel(256);

    // several maps - avoid cloning of username (remove username from state -> username lookup ?)
    let mut entity_positions = std::collections::HashMap::new();
    let mut entity_username = std::collections::HashMap::new();

    let pos_and_look_update_tx_inner = pos_and_look_update_tx.clone();
    let tx_destroy_entities_inner = tx_destroy_entities.clone();
    tokio::spawn(async move {
        loop {
            // receive position updates, log in (username)
            if let Ok((eid, pos_and_look, username)) = pos_and_look_rx.try_recv() {
                let prev_pos_and_look = entity_positions.insert(eid, pos_and_look);
                if let Some(username) = username {
                    entity_username.insert(eid, username);
                }

                // if a player logs in (prev pos is none), not moving entities should be sent
                if prev_pos_and_look.is_none() {
                    for (eid, pos_and_look) in &entity_positions {
                        pos_and_look_update_tx_inner
                            .send((
                                *eid,
                                entities::Type::Player(entity_username[eid].clone()),
                                *pos_and_look,
                                None,
                            ))
                            .unwrap();
                    }
                }

                pos_and_look_update_tx_inner
                    .send((
                        eid,
                        entities::Type::Player(entity_username[&eid].clone()),
                        pos_and_look,
                        prev_pos_and_look,
                    ))
                    .unwrap();
            }

            if let Ok(eid) = rx_entity_destroy.try_recv() {
                entity_positions.remove(&eid);
                entity_username.remove(&eid);

                tx_destroy_entities_inner.send(eid).unwrap();
            }
            tokio::time::sleep(std::time::Duration::from_secs_f64(0.0001)).await;
        }
    });

    loop {
        let mut channels = Channels {
            tx_player_pos_and_look: pos_and_look_tx.clone(),
            rx_entity_movement: pos_and_look_update_tx.clone().subscribe(),
            tx_destroy_self_entity: tx_destroy_self_entity.clone(),
            rx_destroy_entities: tx_destroy_entities.clone().subscribe(),
        };

        let stream = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let rx_entity_movement = &mut channels.rx_entity_movement;
            let rx_destroy_entities = &mut channels.rx_destroy_entities;

            // used to clear the prevoius buffered moves ..
            while rx_entity_movement.try_recv().err() != Some(TryRecvError::Empty) {}
            while rx_destroy_entities.try_recv().err() != Some(TryRecvError::Empty) {}

            handle_client(stream.0, chunks, channels).await;
        });
    }
}

pub struct Channels {
    tx_player_pos_and_look: mpsc::Sender<(i32, PositionAndLook, Option<String>)>,
    rx_entity_movement: broadcast::Receiver<(
        i32,
        entities::Type,
        PositionAndLook,
        Option<PositionAndLook>,
    )>,
    tx_destroy_self_entity: mpsc::Sender<i32>,
    rx_destroy_entities: broadcast::Receiver<i32>,
}

const SIZE: usize = 1024 * 8;

#[derive(Debug)]
pub struct ClientHandshake {
    username: String,
}

pub enum Error {
    Incomplete,
}

pub async fn keep_alive(
    _buf: &mut Cursor<&[u8]>,
    stream: &mut TcpStream,
) -> Result<(), PacketError> {
    let packet = vec![0];
    stream.write_all(&packet).await.unwrap();
    stream.flush().await.unwrap();
    Ok(())
}

pub async fn send_chunk(chunk: &Chunk, stream: &mut TcpStream) -> Result<(), PacketError> {
    packet::PreChunkPacket {
        x: chunk.chunk_x,
        z: chunk.chunk_z,
        mode: true,
    }
    .send(stream)
    .await?;

    // let mut map_chunk = vec![0x33];
    let x = chunk.chunk_x * 16;
    let y = 0i16;
    let z = chunk.chunk_z * 16;

    let mut to_compress = chunk.blocks.clone();
    to_compress.extend_from_slice(&chunk.data);
    to_compress.extend_from_slice(&chunk.block_light);
    to_compress.extend_from_slice(&chunk.sky_light);

    unsafe {
        let mut len = libz_sys::compressBound(to_compress.len() as u64);
        let mut compressed_bytes = vec![0u8; len as usize];
        libz_sys::compress(
            compressed_bytes.as_mut_ptr(),
            &mut len,
            to_compress.as_ptr(),
            to_compress.len().try_into().unwrap(),
        );

        packet::MapChunkPacket {
            x,
            y,
            z,
            size_x: 15,
            size_y: 127,
            size_z: 15,
            compressed_size: len as i32,
            compressed_data: compressed_bytes[..len as usize].to_vec(),
        }
        .send(stream)
        .await?;
    }

    Ok(())
}
fn get_id() -> i32 {
    static COUNTER: AtomicI32 = AtomicI32::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

// add checking with peak (faster)
async fn parse_packet(
    stream: &mut TcpStream,
    buf: &BytesMut,
    chunks: &[Chunk],
    state: &RwLock<State>,
    entity_tx: &Sender<(i32, PositionAndLook, Option<String>)>,
    tx_disconnect: &Sender<i32>,
    logged_in: &AtomicBool,
) -> Result<usize, PacketError> {
    let mut buf = Cursor::new(&buf[..]);

    // let packet_id = get_u8(&mut buf)?;
    // println!("packet_id: {packet_id}");

    // println!("buf: {buf:?}");

    // some packets may accumulate, therefore process all of them (happened especially for 0x0A)
    while let Ok(packet_id) = get_u8(&mut buf) {
        match packet_id {
            0 => keep_alive(&mut buf, stream).await?,
            1 => {
                let login_request = packet::LoginRequestPacket::nested_deserialize(&mut buf)?;
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

                for chunk in chunks.iter() {
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
                    packet::PlayerInventoryPacket {
                        inventory_type: id,
                        count,
                        payload: vec![(-1i16).to_be_bytes(); count as usize].concat(),
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
                entity_tx
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
            }
            // Handshake
            0x02 => {
                // skip(&mut buf, 1)?;
                let username = get_string(&mut buf)?;
                let ch = ClientHandshake { username };
                stream.write_all(&[2, 0, 1, b'-']).await.unwrap();
                stream.flush().await.unwrap();
                println!("ch: {ch:?}");
            }
            0x03 => {
                let message = get_string(&mut buf)?;
                println!("{message}")
            }
            0x0A => {
                let _on_ground = get_u8(&mut buf)? != 0;
                // println!("on_ground: {on_ground}");
            }

            0x0B => {
                packet::PlayerPositionPacket::nested_deserialize(&mut buf)?;
                let x = get_f64(&mut buf)?;
                let y = get_f64(&mut buf)?;
                let _stance = get_f64(&mut buf)?;
                let z = get_f64(&mut buf)?;
                let _on_ground = get_u8(&mut buf)? != 0;

                let outer_state;
                {
                    let mut state = state.write().await;

                    state.position_and_look.x = x;
                    state.position_and_look.y = y;
                    state.position_and_look.z = z;
                    outer_state = (state.entity_id, state.position_and_look);
                }
                entity_tx
                    .send((outer_state.0, outer_state.1, None))
                    .await
                    .unwrap();
                // println!("{x} {y} {stance} {z} {on_ground}");
            }

            0x0C => {
                let yaw = get_f32(&mut buf)?;
                let pitch = get_f32(&mut buf)?;
                let _on_ground = get_u8(&mut buf)? != 0;

                let outer_state;
                {
                    let mut state = state.write().await;
                    state.position_and_look.yaw = yaw;
                    state.position_and_look.pitch = pitch;
                    outer_state = (state.entity_id, state.position_and_look);
                }
                entity_tx
                    .send((outer_state.0, outer_state.1, None))
                    .await
                    .unwrap();
                // println!("{yaw} {pitch} {on_ground}");
            }

            0x0D => {
                let x = get_f64(&mut buf)?;
                let y = get_f64(&mut buf)?;
                let _stance = get_f64(&mut buf)?;
                let z = get_f64(&mut buf)?;
                let yaw = get_f32(&mut buf)?;
                let pitch = get_f32(&mut buf)?;
                let _on_ground = get_u8(&mut buf)? != 0;
                let outer_state;
                {
                    let mut state = state.write().await;
                    state.position_and_look.x = x;
                    state.position_and_look.y = y;
                    state.position_and_look.z = z;
                    state.position_and_look.yaw = yaw;
                    state.position_and_look.pitch = pitch;
                    outer_state = (state.entity_id, state.position_and_look);
                }
                entity_tx
                    .send((outer_state.0, outer_state.1, None))
                    .await
                    .unwrap();

                // println!("{x} {y} {stance} {z} {yaw} {pitch} {on_ground}");
            }
            0x12 => {
                let pid = get_i32(&mut buf)?;
                let arm_winging = get_u8(&mut buf)? > 0;
                println!("{pid} {arm_winging}")
            }
            0xff => {
                // player.should_disconnect = true;
                let reason = get_string(&mut buf)?;
                println!("disconnect: {reason}");
                tx_disconnect
                    .send(state.read().await.entity_id)
                    .await
                    .unwrap();
            }
            _ => {
                println!("packet_id: {packet_id}");
                return Err(PacketError::NotEnoughBytes);
            }
        }
    }
    Ok(buf.position() as usize)
}

pub struct State {
    entity_id: i32,
    username: String,
    logged_in: bool,
    position_and_look: PositionAndLook,
}

#[derive(Debug, Clone, Copy)]
pub struct PositionAndLook {
    x: f64,
    y: f64,
    z: f64,
    yaw: f32,
    pitch: f32,
}

async fn handle_client(stream: TcpStream, chunks: &[Chunk], channels: Channels) {
    let mut buf = BytesMut::with_capacity(SIZE);

    let Channels {
        tx_player_pos_and_look,
        rx_entity_movement,
        tx_destroy_self_entity,
        rx_destroy_entities,
    } = channels;

    let stream = Arc::new(RwLock::new(stream));
    let keep_alive_stream = stream.clone();

    let state = Arc::new(RwLock::new(State {
        entity_id: 0,
        username: "".to_string(),
        logged_in: false,
        position_and_look: PositionAndLook {
            x: 0.,
            y: 0.,
            z: 0.,
            yaw: 0.,
            pitch: 0.,
        },
    }));

    let logged_in = Arc::new(AtomicBool::new(false));

    // spawn or update entities
    tokio::task::spawn(global_handlers::spawn_entities(
        logged_in.clone(),
        state.clone(),
        rx_entity_movement,
        stream.clone(),
    ));

    // destroy entities
    tokio::task::spawn(global_handlers::destroy_entities(
        rx_destroy_entities,
        stream.clone(),
    ));

    tokio::task::spawn(async move {
        loop {
            let packet = vec![0];
            keep_alive_stream
                .write()
                .await
                .write_all(&packet)
                .await
                .unwrap();
            keep_alive_stream.write().await.flush().await.unwrap();

            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    });

    loop {
        if let Ok(n) = parse_packet(
            &mut *stream.write().await,
            &buf,
            chunks,
            &state,
            &tx_player_pos_and_look,
            &tx_destroy_self_entity,
            &logged_in,
        )
        .await
        {
            buf.advance(n);
        }

        if stream.write().await.read_buf(&mut buf).await.unwrap() == 0 {
            println!("break");
            break;
        }

        // println!("{player:?}")
    }

    tx_destroy_self_entity
        .send(state.read().await.entity_id)
        .await
        .unwrap();
}
