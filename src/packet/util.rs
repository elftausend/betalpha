use crate::packet::PacketError;
use bytes::Buf;
use std::io::Cursor;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use super::Serialize;

pub trait SendPacket {
    async fn send(&self, stream: &mut TcpStream) -> Result<(), PacketError>;
}

impl<P: Serialize> SendPacket for P {
    #[inline]
    async fn send(&self, stream: &mut TcpStream) -> Result<(), PacketError> {
        stream
            .write_all(&self.serialize()?)
            .await
            .map_err(|e| PacketError::IOError(e))?;

        // consider not flushing
        stream.flush().await.map_err(|e| PacketError::IOError(e))
    }
}

pub fn peek_u8(src: &mut Cursor<&[u8]>) -> Result<u8, PacketError> {
    if !src.has_remaining() {
        return Err(PacketError::NotEnoughBytes);
    }
    Ok(src.chunk()[0])
}

pub fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, PacketError> {
    if !src.has_remaining() {
        return Err(PacketError::NotEnoughBytes);
    }
    Ok(src.get_u8())
}

pub fn get_u16(src: &mut Cursor<&[u8]>) -> Result<u16, PacketError> {
    if src.remaining() < 2 {
        return Err(PacketError::NotEnoughBytes);
    }
    Ok(src.get_u16())
}

pub fn get_i8(src: &mut Cursor<&[u8]>) -> Result<i8, PacketError> {
    if !src.has_remaining() {
        return Err(PacketError::NotEnoughBytes);
    }
    Ok(src.get_i8())
}

pub fn get_u32(src: &mut Cursor<&[u8]>) -> Result<u32, PacketError> {
    if src.remaining() < 4 {
        return Err(PacketError::NotEnoughBytes);
    }
    Ok(src.get_u32())
}

pub fn get_i32(src: &mut Cursor<&[u8]>) -> Result<i32, PacketError> {
    if src.remaining() < 4 {
        return Err(PacketError::NotEnoughBytes);
    }
    Ok(src.get_i32())
}

pub fn get_f32(src: &mut Cursor<&[u8]>) -> Result<f32, PacketError> {
    if src.remaining() < 4 {
        return Err(PacketError::NotEnoughBytes);
    }
    Ok(src.get_f32())
}

pub fn get_f64(src: &mut Cursor<&[u8]>) -> Result<f64, PacketError> {
    if src.remaining() < 8 {
        return Err(PacketError::NotEnoughBytes);
    }
    Ok(src.get_f64())
}

pub fn get_u64(src: &mut Cursor<&[u8]>) -> Result<u64, PacketError> {
    if src.remaining() < 8 {
        return Err(PacketError::NotEnoughBytes);
    }
    Ok(src.get_u64())
}

pub fn get_string(src: &mut Cursor<&[u8]>) -> Result<String, PacketError> {
    let len = get_u16(src)?;
    if src.remaining() < len as usize {
        return Err(PacketError::NotEnoughBytes);
    }
    let string = String::from_utf8(src.chunk()[..len as usize].to_vec())
        .map_err(|_e| PacketError::InvalidString)?;
    skip(src, len as usize)?;
    Ok(string)
}

pub fn get_inventory_payload(_src: &mut Cursor<&[u8]>) -> Result<Vec<u8>, PacketError> {
    todo!()
}

pub fn skip(src: &mut Cursor<&[u8]>, n: usize) -> Result<(), PacketError> {
    if src.remaining() < n {
        return Err(PacketError::NotEnoughBytes);
    }
    src.advance(n);
    Ok(())
}

pub fn string_to_bytes(string: String) -> Vec<u8> {
    [
        (string.len() as u16).to_be_bytes().as_slice(),
        string.into_bytes().as_slice(),
    ]
    .concat()
}
