use tokio::net::TcpStream;

use crate::packet::{self, util::SendPacket, PacketError, PlayerBlockPlacementPacket};

pub mod load_demo;

#[derive(Debug, Clone, Copy)]
pub enum BlockUpdate {
    Place(PlayerBlockPlacementPacket),
    Break((i32, i8, i32)),
}

pub struct Chunk {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub blocks: Vec<u8>,
    pub data: Vec<u8>,
    pub sky_light: Vec<u8>,
    pub block_light: Vec<u8>,
    pub height_map: Vec<u8>,
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
        let mut len = libz_sys::compressBound(to_compress.len().try_into().unwrap());
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
