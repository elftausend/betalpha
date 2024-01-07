mod packet;

use std::{io::{Cursor, Read, Write}, sync::Arc};
use std::{
    collections::HashSet,
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicBool, AtomicI32, Ordering},
};

use bytes::{Buf, BytesMut};

use nbt::{Blob, Value};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{
        broadcast::{self},
        mpsc::{self, Sender},
        RwLock,
    },
};
use crate::packet::PacketError;
use crate::packet::util::*;

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

fn test_libz() {
    // libz_sys::compress(dest, destLen, source, sourceLen)
    // Original data
    let original_data = b"Hello, world!";
    /*
    // Initialize the z_stream structure
    let mut stream = z_stream {
        next_in: original_data.as_ptr() as *mut _,
        avail_in: original_data.len() as u32,
        ..Default::default()
    };

    // Initialize the deflate stream
    let mut ret = unsafe { deflateInit_(&mut stream, libz_sys::Z_DEFAULT_COMPRESSION) };
    assert_eq!(ret, Z_OK);

    // Buffer to hold compressed data
    let mut compressed_data = vec![0u8; original_data.len() * 2];

    // Set the output buffer
    stream.next_out = compressed_data.as_mut_ptr() as *mut _;
    stream.avail_out = compressed_data.len() as u32;

    // Perform the compression
    ret = unsafe { deflate(&mut stream, libz_sys::Z_FINISH) };
    assert_eq!(ret, Z_STREAM_END);

    // Clean up the deflate stream
    ret = unsafe { deflateEnd(&mut stream) };
    assert_eq!(ret, Z_OK);

    // Resize the compressed_data vector to the actual size
    compressed_data.resize((stream.next_out as usize - compressed_data.as_ptr() as usize) / std::mem::size_of::<u8>(), 0);

    println!("Compressed data: {:?}", compressed_data);*/
}

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
pub async fn incomplete(buf: &mut Cursor<&[u8]>, stream: &mut TcpStream) -> Result<(), Error> {
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
    let (pos_and_look_tx, mut pos_and_look_rx) = mpsc::channel::<(i32, PositionAndLook)>(256);

    let (pos_and_look_update_tx, mut pos_and_look_update_rx) = broadcast::channel(256);

    let mut entity_positions = std::collections::HashMap::new();
    // let mut entity

    let pos_and_look_update_tx_inner = pos_and_look_update_tx.clone();
    tokio::spawn(async move {
        loop {
            if let Some((eid, pos_and_look)) = pos_and_look_rx.recv().await {
                let prev_pos_and_look = entity_positions.insert(eid, pos_and_look);

                // if a player logs in, not moving entities should be sent
                if prev_pos_and_look.is_none() {
                    for (eid, pos_and_look) in &entity_positions {
                        pos_and_look_update_tx_inner
                            .send((*eid, entities::Type::Player, *pos_and_look, None))
                            .unwrap();
                    }
                }

                pos_and_look_update_tx_inner
                    .send((eid, entities::Type::Player, pos_and_look, prev_pos_and_look))
                    .unwrap();
            }
        }
    });

    loop {
        let mut channels = Channels {
            tx_player_pos_and_look: pos_and_look_tx.clone(),
            rx_entity_movement: pos_and_look_update_tx.clone().subscribe(),
        };

        let stream = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let rx_entity_movement = &mut channels.rx_entity_movement;
            // used to clear the prevoius buffered moves
            while let Ok(_) = rx_entity_movement.try_recv() {}
            handle_client(stream.0, chunks, channels).await;
        });
    }
}

pub struct Channels {
    tx_player_pos_and_look: mpsc::Sender<(i32, PositionAndLook)>,
    rx_entity_movement: broadcast::Receiver<(
        i32,
        entities::Type,
        PositionAndLook,
        Option<PositionAndLook>,
    )>,
}

const SIZE: usize = 1024 * 8;

#[derive(Debug)]
pub struct ClientHandshake {
    username: String,
}

pub enum Error {
    Incomplete,
}

pub async fn keep_alive(_buf: &mut Cursor<&[u8]>, stream: &mut TcpStream) -> Result<(), PacketError> {
    let packet = vec![0];
    stream.write_all(&packet).await.unwrap();
    stream.flush().await.unwrap();
    Ok(())
}

pub async fn send_chunk(chunk: &Chunk, stream: &mut TcpStream) -> tokio::io::Result<()> {
    let mut pre_chunk = vec![0x32];
    pre_chunk.extend_from_slice(&chunk.chunk_x.to_be_bytes());
    pre_chunk.extend_from_slice(&chunk.chunk_z.to_be_bytes());
    pre_chunk.extend_from_slice(&[1u8]);

    stream.write_all(&pre_chunk).await?;
    stream.flush().await?;

    let mut map_chunk = vec![0x33];
    let x = chunk.chunk_x * 16;
    let y = 0i16;
    let z = chunk.chunk_z * 16;

    map_chunk.extend_from_slice(&x.to_be_bytes());
    map_chunk.extend_from_slice(&y.to_be_bytes());
    map_chunk.extend_from_slice(&z.to_be_bytes());

    map_chunk.extend_from_slice(&15u8.to_be_bytes());
    map_chunk.extend_from_slice(&127u8.to_be_bytes());
    map_chunk.extend_from_slice(&15u8.to_be_bytes());

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
            to_compress.len() as u64,
        );

        map_chunk.extend_from_slice(&(len as i32).to_be_bytes());
        map_chunk.extend_from_slice(&compressed_bytes[..len as usize]);
    }

    stream.write_all(&map_chunk).await?;
    stream.flush().await
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
    entity_tx: &Sender<(i32, PositionAndLook)>,
    logged_in: &AtomicBool,
) -> Result<usize, PacketError> {
    let mut buf = Cursor::new(&buf[..]);

    let packet_id = get_u8(&mut buf)?;
    // println!("packet_id: {packet_id}");

    // println!("buf: {buf:?}");

    match packet_id {
        0 => keep_alive(&mut buf, stream).await?,
        1 => {
            let protocol_version = get_i32(&mut buf)?;
            // skip(&mut buf, 1)?;
            let username = get_string(&mut buf)?;
            // skip(&mut buf, 1)?;
            let _password = get_string(&mut buf)?;
            let _map_seed = get_u64(&mut buf)?;
            let _dimension = get_i8(&mut buf)?;

            let entity_id = get_id();
            // let seed = 1111423422i64;
            let seed: i64 = 9065250152070435348;
            // let seed: i64 = -4264101711260417039;
            let dimension = 0i8; // -1 hell

            let mut packet = vec![1];
            packet.extend_from_slice(&entity_id.to_be_bytes());

            packet.extend_from_slice(&[0, 0, 0, 0]);
            #[rustfmt::skip]
            // packet.extend_from_slice(&[0, 0,0, 0, 0,0, 0]);
            packet.extend_from_slice(&seed.to_be_bytes());
            // packet.extend_from_slice(&[0 ]);
            packet.extend_from_slice(&dimension.to_be_bytes());

            stream.write_all(&packet).await.unwrap();
            stream.flush().await.unwrap();

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

            // -56.277393,65.62,70.65869
            let x = -56i32;
            let y = 80i32;
            let z = 70i32;
            let mut spawn_position = vec![0x06];
            spawn_position.extend_from_slice(&x.to_be_bytes());
            spawn_position.extend_from_slice(&y.to_be_bytes());
            spawn_position.extend_from_slice(&z.to_be_bytes());

            stream.write_all(&spawn_position).await.unwrap();
            stream.flush().await.unwrap();

            println!("sent spawn");

            for (id, count) in [(-1i32, 36i16), (-2, 4), (-3, 4)] {
                let mut player_inventory = vec![0x05];
                player_inventory.extend_from_slice(&id.to_be_bytes());
                player_inventory.extend_from_slice(&count.to_be_bytes());
                for _ in 0..count {
                    player_inventory.extend_from_slice(&(-1i16).to_be_bytes());
                }

                stream.write_all(&player_inventory).await.unwrap();
                stream.flush().await.unwrap();
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
                outer_state = (state.entity_id, state.position_and_look);
            }
            entity_tx
                .send((outer_state.0, outer_state.1))
                .await
                .unwrap();

            let on_ground = true;

            let mut position_look = vec![0x0D];
            position_look.extend_from_slice(&x.to_be_bytes());
            // mind stance order
            position_look.extend_from_slice(&stance.to_be_bytes());
            position_look.extend_from_slice(&y.to_be_bytes());
            position_look.extend_from_slice(&z.to_be_bytes());

            position_look.extend_from_slice(&yaw.to_be_bytes());
            position_look.extend_from_slice(&pitch.to_be_bytes());
            position_look.extend_from_slice(&[on_ground as u8]);

            stream.write_all(&position_look).await.unwrap();
            stream.flush().await.unwrap();
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
            let on_ground = get_u8(&mut buf)? != 0;
            // println!("on_ground: {on_ground}");
        }

        0x0B => {
            let x = get_f64(&mut buf)?;
            let y = get_f64(&mut buf)?;
            let stance = get_f64(&mut buf)?;
            let z = get_f64(&mut buf)?;
            let on_ground = get_u8(&mut buf)? != 0;

            let outer_state;
            {
                let mut state = state.write().await;

                state.position_and_look.x = x;
                state.position_and_look.y = y;
                state.position_and_look.z = z;
                outer_state = (state.entity_id, state.position_and_look);
            }
            entity_tx
                .send((outer_state.0, outer_state.1))
                .await
                .unwrap();
            // println!("{x} {y} {stance} {z} {on_ground}");
        }

        0x0C => {
            let yaw = get_f32(&mut buf)?;
            let pitch = get_f32(&mut buf)?;
            let on_ground = get_u8(&mut buf)? != 0;

            let outer_state;
            {
                let mut state = state.write().await;
                state.position_and_look.yaw = yaw;
                state.position_and_look.pitch = pitch;
                outer_state = (state.entity_id, state.position_and_look);
            }
            entity_tx
                .send((outer_state.0, outer_state.1))
                .await
                .unwrap();
            // println!("{yaw} {pitch} {on_ground}");
        }

        0x0D => {
            let x = get_f64(&mut buf)?;
            let y = get_f64(&mut buf)?;
            let stance = get_f64(&mut buf)?;
            let z = get_f64(&mut buf)?;
            let yaw = get_f32(&mut buf)?;
            let pitch = get_f32(&mut buf)?;
            let on_ground = get_u8(&mut buf)? != 0;
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
                .send((outer_state.0, outer_state.1))
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
            println!("{reason}")
        }
        _ => {
            println!("packet_id: {packet_id}");
            return Err(PacketError::NotEnoughBytes)
        },
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
        mut rx_entity_movement,
    } = channels;

    let stream = Arc::new(RwLock::new(stream));
    let keep_alive_stream = stream.clone();
    let pos_update_stream = stream.clone();

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

    let logged_in_inner = logged_in.clone();
    let state_pos_update = state.clone();
    tokio::task::spawn(async move {
        let mut seen_before = HashSet::new();
        loop {
            // single core servers
            // tokio::time::sleep(std::time::Duration::from_secs_f64(0.001)).await;
            if !logged_in_inner.load(Ordering::Relaxed) {
                continue;
            }
            let Ok((eid, ty, now, prev)) = rx_entity_movement.recv().await else {
                continue;
            };

            if eid == state_pos_update.read().await.entity_id {
                continue;
            }

            // TODO: add eid is in reach check, unload/destroy entity
            // FIXME: could potentially receive a lot of data / entity information that is intantly discarded

            // println!(
            //     "i am: {}, moved: {eid} {now:?}, prev: {prev:?}",
            //     state_pos_update.read().await.entity_id
            // );

            if !seen_before.contains(&eid) {
                let mut pos_update_stream = pos_update_stream.write().await;

                match ty {
                    entities::Type::Player => {
                        spawned_named_entity(&mut pos_update_stream, eid, "Stefan", &now).await
                    }
                };

                let mut entity_spawn = vec![0x1E];
                entity_spawn.extend_from_slice(&eid.to_be_bytes());

                pos_update_stream.write_all(&entity_spawn).await.unwrap();
                pos_update_stream.flush().await.unwrap();
            }

            seen_before.insert(eid);

            if let Some(prev) = prev {
                // check if travelled blocks is > 4 (teleport)

                let x = ((now.x - prev.x) * 32.).round() as i8;
                let y = ((now.y - prev.y) * 32.).round() as i8;
                let z = ((now.z - prev.z) * 32.).round() as i8;
                let yawf = ((now.yaw / 360.) * 255.) % 255.;
                let pitch = (((now.pitch / 360.) * 255.) % 255.) as i8;

                let mut yaw = yawf as i8;
                if yawf < -128. {
                    yaw = 127 - (yawf + 128.).abs() as i8
                }
                if yawf > 128. {
                    yaw = -128 + (yawf - 128.).abs() as i8
                }

                // println!("yaw: {yawf} {} pitch: {pitch}", yaw);

                let mut entity_look_and_move = vec![0x21];
                entity_look_and_move.extend_from_slice(&eid.to_be_bytes());
                entity_look_and_move.extend_from_slice(&[
                    x.to_be_bytes()[0],
                    y.to_be_bytes()[0],
                    z.to_be_bytes()[0],
                    yaw.to_be_bytes()[0],
                    pitch.to_be_bytes()[0],
                ]);

                let mut pos_update_stream = pos_update_stream.write().await;
                pos_update_stream
                    .write_all(&entity_look_and_move)
                    .await
                    .unwrap();
                pos_update_stream.flush().await.unwrap();
            }
        }
    });

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
}
