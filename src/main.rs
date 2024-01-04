#![feature(read_buf)]
use std::{
    collections::HashMap,
    io::{Cursor, Read, Write},
};

use bytes::{Buf, BytesMut};
use flate2::{
    read::GzDecoder,
    write::{DeflateEncoder, GzEncoder, ZlibEncoder},
    Compression,
};
use nbt::{Blob, Value};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

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

pub struct Chunk {
    chunk_x: i32,
    chunk_z: i32,
    blocks: Vec<u8>,
    data: Vec<u8>,
    sky_light: Vec<u8>,
    block_light: Vec<u8>,
    height_map: Vec<u8>,
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:25565").await.unwrap();

    // let mut chunks = Vec::new();
    let dirs = walkdir::WalkDir::new("/home/elftausend/.minecraft/saves/World1/")
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

            let Value::Compound(level) = &blob["Level"] else {
                panic!()
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

const SIZE: usize = 100;

#[derive(Debug)]
pub struct ClientHandshake {
    username: String,
}

#[derive(Debug, Clone, Copy)]
enum ParseRule {
    I32,
    String,
    I8,
    U8,
}

pub enum Error {
    Incomplete,
}

fn peek_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }

    Ok(src.chunk()[0])
}

fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }
    Ok(src.get_u8())
}

fn get_i8(src: &mut Cursor<&[u8]>) -> Result<i8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }
    Ok(src.get_i8())
}

fn get_i32(src: &mut Cursor<&[u8]>) -> Result<i32, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }
    Ok(src.get_i32())
}

fn get_string(src: &mut Cursor<&[u8]>) -> Result<String, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }

    let len = get_u8(src)?;
    let string = String::from_utf8_lossy(&src.chunk()[..len as usize]).to_string();
    skip(src, len as usize)?;
    Ok(string)
}

fn skip(src: &mut Cursor<&[u8]>, n: usize) -> Result<(), Error> {
    if src.remaining() < n {
        return Err(Error::Incomplete);
    }

    src.advance(n);
    Ok(())
}

// add checking with peak (faster)
async fn parse_packet(
    stream: &mut TcpStream,
    buf: &BytesMut,
    chunks: &[Chunk],
) -> Result<usize, Error> {
    let mut buf = Cursor::new(&buf[..]);

    let packet_id = get_u8(&mut buf)?;
    println!("packet_id: {packet_id}");

    println!("buf: {buf:?}");

    match packet_id {
        0 => {
            let packet = vec![0];
            stream.write_all(&packet).await.unwrap();
            stream.flush().await.unwrap();
        }
        1 => {
            let protocol_version = get_i32(&mut buf)?;
            skip(&mut buf, 1)?;
            let username = get_string(&mut buf)?;
            skip(&mut buf, 1)?;
            let _password = get_string(&mut buf)?;

            let entity_id = 1337i32;
            // let seed = 1111423422i64;
            let seed: i64 = -4264101711260417039;
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
                // to_compress.extend_from_slice(&chunk.height_map);

                assert_eq!(to_compress.len(), ((16 * 128 * 16) as f32 * 2.5) as usize);

                // let mut e = DeflateEncoder::new(Vec::new(), Compression::default());
                // e.write_all(&to_compress).unwrap();

                // let compressed_bytes = e.finish().unwrap();

                // let compressed_bytes = deflate::deflate_bytes_conf(&to_compress, deflate::Compression::Fast);
                // map_chunk.extend_from_slice(&[0 ]);
                let compressed_bytes = deflate::deflate_bytes(&to_compress);
                map_chunk.extend_from_slice(&(compressed_bytes.len() as i32).to_be_bytes());
                map_chunk.extend_from_slice(&compressed_bytes);

                stream.write_all(&map_chunk).await.unwrap();
                stream.flush().await.unwrap();
            }
            println!("sent map");

            // -56.277393,65.62,70.65869
            let x = -56i32;
            let y = 65i32;
            let z = 70i32;
            let mut spawn_position = vec![0x06];
            spawn_position.extend_from_slice(&x.to_be_bytes());
            spawn_position.extend_from_slice(&y.to_be_bytes());
            spawn_position.extend_from_slice(&z.to_be_bytes());

            stream.write_all(&spawn_position).await.unwrap();
            stream.flush().await.unwrap();

            println!("sent spawn");

            for id in 1..=3 {
                let id: i32 = -1 * id;
                let count: i16 = 0;
                let mut player_inventory = vec![0x05];
                player_inventory.extend_from_slice(&id.to_be_bytes());
                player_inventory.extend_from_slice(&count.to_be_bytes());

                stream.write_all(&player_inventory).await.unwrap();
                stream.flush().await.unwrap();
            }

            println!("sent inv");

            let x = -56.27f64;
            let y = 65.62f64;
            let z = 70.65f64;
            let stance: f64 = y + 1.6;

            let yaw = 0f32;
            let pitch = 0f32;

            let on_ground = false;

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
        2 => {
            skip(&mut buf, 1)?;
            let username = get_string(&mut buf)?;
            let ch = ClientHandshake { username };
            stream.write_all(&[2, 0, 1, b'-']).await.unwrap();
            stream.flush().await.unwrap();
            println!("ch: {ch:?}");
        }
        _ => {}
    }

    Ok(buf.position() as usize)
}

async fn handle_client(mut stream: TcpStream, chunks: &[Chunk]) {
    let mut buf = BytesMut::with_capacity(SIZE);

    loop {
        if let Ok(n) = parse_packet(&mut stream, &buf, chunks).await {
            buf.advance(n);
            buf.clear(); // some fields in packets are ignored
        }
        if stream.read_buf(&mut buf).await.unwrap() == 0 {
            println!("break");
            break;
        }
    }
}
