mod packet;

use std::{io::{Cursor, Read, Write}, sync::Arc};

use bytes::{Buf, BytesMut};


use nbt::{Blob, Value};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream}, sync::RwLock,
};
use crate::packet::PacketError;
use crate::packet::util::*;

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

extern crate libz_sys;

pub struct Chunk {
    chunk_x: i32,
    chunk_z: i32,
    blocks: Vec<u8>,
    data: Vec<u8>,
    sky_light: Vec<u8>,
    block_light: Vec<u8>,
    height_map: Vec<u8>,
}

#[derive(Debug)]
pub struct Player {
    protocol_version: i32,
    logged_in: bool,
    should_disconnect: bool,
    username: String,
    x: f64,
    y: f64,
    z: f64,
    stance: f64,
    on_ground: bool,
    yaw: f32,
    pitch: f32,
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:25565").await.unwrap();

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

    loop {
        let stream = listener.accept().await.unwrap();
        tokio::spawn(async move {
            handle_client(stream.0, chunks).await;
        });
    }
}

const SIZE: usize = 1024 * 8;

#[derive(Debug)]
pub struct ClientHandshake {
    username: String,
}

// add checking with peak (faster)
async fn parse_packet(
    stream: &mut TcpStream,
    buf: &BytesMut,
    chunks: &[Chunk],
    player: &mut Player,
) -> Result<usize, PacketError> {
    let mut buf = Cursor::new(&buf[..]);

    let packet_id = get_u8(&mut buf)?;

    // println!("buf: {buf:?}");

    // remove later
    // if *logged_in {
    //     return Ok(buf.remaining() as usize);
    // }

    match packet_id {
        // Keep Alive
        0x00 => {
            let packet = vec![0];
            stream.write_all(&packet).await.unwrap();
            stream.flush().await.unwrap();
        }
        // Login Request
        0x01 => {
            let protocol_version = get_i32(&mut buf)?;
            // skip(&mut buf, 1)?;
            let username = get_string(&mut buf)?;
            // skip(&mut buf, 1)?;
            let _password = get_string(&mut buf)?;
            let _map_seed = get_u64(&mut buf)?;
            let _dimension = get_i8(&mut buf)?;

            let entity_id = 1337i32;
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

            player.protocol_version = protocol_version;
            player.username = username;

            player.logged_in = true;
            for chunk in chunks.iter() {
                let mut pre_chunk = vec![0x32];
                pre_chunk.extend_from_slice(&chunk.chunk_x.to_be_bytes());
                pre_chunk.extend_from_slice(&chunk.chunk_z.to_be_bytes());
                pre_chunk.extend_from_slice(&[1u8]);

                stream.write_all(&pre_chunk).await.unwrap();
                stream.flush().await.unwrap();

                let mut map_chunk = vec![0x33];
                let x = chunk.chunk_x * 16;
                let y = 0i16;
                let z = chunk.chunk_z * 16;
                // println!("cx: {}, cz: {}, x: {x}, z: {z}", chunk.chunk_x, chunk.chunk_z);
                // map_chunk.extend_from_slice(&[0 ]);
                map_chunk.extend_from_slice(&x.to_be_bytes());
                map_chunk.extend_from_slice(&y.to_be_bytes());
                map_chunk.extend_from_slice(&z.to_be_bytes());

                map_chunk.extend_from_slice(&15u8.to_be_bytes());
                map_chunk.extend_from_slice(&127u8.to_be_bytes());
                map_chunk.extend_from_slice(&15u8.to_be_bytes());
                // println!("map_chunk: {map_chunk:?}");

                let mut to_compress = chunk.blocks.clone();
                to_compress.extend_from_slice(&chunk.data);
                to_compress.extend_from_slice(&chunk.block_light);
                to_compress.extend_from_slice(&chunk.sky_light);

                assert_eq!(to_compress.len(), ((16 * 128 * 16) as f32 * 2.5) as usize);

                // let mut e = DeflateEncoder::new(Vec::new(), Compression::default());
                // e.write_all(&to_compress).unwrap();

                // let compressed_bytes = e.finish().unwrap();

                // let compressed_bytes = deflate::deflate_bytes_conf(&to_compress, deflate::Compression::Fast);
                // map_chunk.extend_from_slice(&[0 ]);
                // let compressed_bytes = deflate::deflate_bytes(&to_compress);

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

                // let data = std::fs::read("compressed_c_bytes").unwrap();
                // let compressed_bytes = data.split(' ').map(|x| x.parse::<i8>().unwrap().to_be_bytes()[0]).collect::<Vec<_>>();
                // let compressed_bytes = data;

                stream.write_all(&map_chunk).await.unwrap();
                stream.flush().await.unwrap();
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
            let y = 80.62f64;
            let z = 0.65f64;
            let stance: f64 = y + 1.6;

            let yaw = 0f32;
            let pitch = 0f32;

            let on_ground = true;

            let mut position_look = vec![0x0D];
            position_look.extend_from_slice(&x.to_be_bytes());
            position_look.extend_from_slice(&y.to_be_bytes());
            position_look.extend_from_slice(&stance.to_be_bytes());
            position_look.extend_from_slice(&z.to_be_bytes());

            position_look.extend_from_slice(&yaw.to_be_bytes());
            position_look.extend_from_slice(&pitch.to_be_bytes());
            position_look.extend_from_slice(&[on_ground as u8]);

            stream.write_all(&position_look).await.unwrap();
            stream.flush().await.unwrap();
            println!("sent pos");
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
            player.on_ground = get_u8(&mut buf)? != 0;
        }

        0x0B => {
            player.x = get_f64(&mut buf)?;
            player.y = get_f64(&mut buf)?;
            player.stance = get_f64(&mut buf)?;
            player.z = get_f64(&mut buf)?;
            player.on_ground = get_u8(&mut buf)? != 0;
        }

        0x0C => {
            player.yaw = get_f32(&mut buf)?;
            player.pitch = get_f32(&mut buf)?;
            player.on_ground = get_u8(&mut buf)? != 0;
        }

        0x0D => {
            player.x = get_f64(&mut buf)?;
            player.y = get_f64(&mut buf)?;
            player.stance = get_f64(&mut buf)?;
            player.z = get_f64(&mut buf)?;
            player.yaw = get_f32(&mut buf)?;
            player.pitch = get_f32(&mut buf)?;
            player.on_ground = get_u8(&mut buf)? != 0;
        }
        0x12 => {
            let pid = get_i32(&mut buf)?;
            let arm_winging = get_u8(&mut buf)? > 0;
            println!("{pid} {arm_winging}")
        }
        0xff => {
            player.should_disconnect = true;
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

async fn handle_client(mut stream: TcpStream, chunks: &[Chunk]) {
    let mut buf = BytesMut::with_capacity(SIZE);

    let stream = Arc::new(RwLock::new(stream));
    let keep_alive_stream = stream.clone();
    let mut player = Player {
        protocol_version: 0,
        logged_in: false,
        should_disconnect: false,
        username: "".to_string(),
        x: 0.0,
        y: 0.0,
        z: 0.0,
        stance: 0.0,
        on_ground: false,
        yaw: 0.0,
        pitch: 0.0,
    };

    tokio::task::spawn(async move {
        loop {
            let packet = vec![0];
            keep_alive_stream.write().await.write_all(&packet).await.unwrap();
            keep_alive_stream.write().await.flush().await.unwrap();
        
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    });
        

    loop {
        if let Ok(n) = parse_packet(&mut *stream.write().await, &buf, chunks, &mut player).await {
            buf.advance(n);
        }

        if stream.write().await.read_buf(&mut buf).await.unwrap() == 0 {
            println!("break");
            break;
        }

        println!("{player:?}")
    }
}
