use std::io::Cursor;
use bytes::Buf;
use crate::packet::ParsingError;

pub fn peek_u8(src: &mut Cursor<&[u8]>) -> Result<u8, ParsingError> {
    if !src.has_remaining() {
        return Err(ParsingError::NotEnoughBytes);
    }
    Ok(src.chunk()[0])
}

pub fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, ParsingError> {
    if !src.has_remaining() {
        return Err(ParsingError::NotEnoughBytes);
    }
    Ok(src.get_u8())
}

pub fn get_u16(src: &mut Cursor<&[u8]>) -> Result<u16, ParsingError> {
    if src.remaining() < 4 {
        return Err(ParsingError::NotEnoughBytes);
    }
    Ok(src.get_u16())
}

pub fn get_i8(src: &mut Cursor<&[u8]>) -> Result<i8, ParsingError> {
    if !src.has_remaining() {
        return Err(ParsingError::NotEnoughBytes);
    }
    Ok(src.get_i8())
}

pub fn get_u32(src: &mut Cursor<&[u8]>) -> Result<u32, ParsingError> {
    if src.remaining() < 4 {
        return Err(ParsingError::NotEnoughBytes);
    }
    Ok(src.get_u32())
}

pub fn get_i32(src: &mut Cursor<&[u8]>) -> Result<i32, ParsingError> {
    if src.remaining() < 4 {
        return Err(ParsingError::NotEnoughBytes);
    }
    Ok(src.get_i32())
}

pub fn get_f32(src: &mut Cursor<&[u8]>) -> Result<f32, ParsingError> {
    if src.remaining() < 4 {
        return Err(ParsingError::NotEnoughBytes);
    }
    Ok(src.get_f32())
}

pub fn get_f64(src: &mut Cursor<&[u8]>) -> Result<f64, ParsingError> {
    if src.remaining() < 8 {
        return Err(ParsingError::NotEnoughBytes);
    }
    Ok(src.get_f64())
}

pub fn get_u64(src: &mut Cursor<&[u8]>) -> Result<u64, ParsingError> {
    if src.remaining() < 8 {
        return Err(ParsingError::NotEnoughBytes);
    }
    Ok(src.get_u64())
}

pub fn get_string(src: &mut Cursor<&[u8]>) -> Result<String, ParsingError> {
    let len = get_u16(src)?;
    if src.remaining() < len as usize {
        return Err(ParsingError::NotEnoughBytes);
    }
    let string = String::from_utf8(src.chunk()[..len as usize].to_vec()).map_err(|_e| ParsingError::InvalidString)?;
    skip(src, len as usize)?;
    Ok(string)
}

pub fn get_inventory_payload(src: &mut Cursor<&[u8]>) -> Result<Vec<u8>, ParsingError> {
    todo!()
}

pub fn skip(src: &mut Cursor<&[u8]>, n: usize) -> Result<(), ParsingError> {
    if src.remaining() < n {
        return Err(ParsingError::NotEnoughBytes);
    }
    src.advance(n);
    Ok(())
}