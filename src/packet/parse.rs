use crate::packet::PacketError;
use bytes::Buf;
use std::io::Cursor;

#[derive(Default)]
pub struct PacketSerializer {
    pub output: Vec<u8>,
}

impl PacketSerializer {
    pub fn serialize_bool(&mut self, v: bool) -> Result<(), PacketError> {
        self.output
            .append(&mut [if v { 0x01 } else { 0x00 }].to_vec());
        Ok(())
    }

    pub fn serialize_u8(&mut self, v: u8) -> Result<(), PacketError> {
        self.output.extend_from_slice(&v.to_be_bytes());
        Ok(())
    }

    pub fn serialize_u16(&mut self, v: u16) -> Result<(), PacketError> {
        self.output.extend_from_slice(&v.to_be_bytes());
        Ok(())
    }

    pub fn serialize_u32(&mut self, v: u32) -> Result<(), PacketError> {
        self.output.extend_from_slice(&v.to_be_bytes());
        Ok(())
    }

    pub fn serialize_u64(&mut self, v: u64) -> Result<(), PacketError> {
        self.output.extend_from_slice(&v.to_be_bytes());
        Ok(())
    }

    pub fn serialize_f32(&mut self, v: f32) -> Result<(), PacketError> {
        self.output.extend_from_slice(&v.to_be_bytes());
        Ok(())
    }

    pub fn serialize_f64(&mut self, v: f64) -> Result<(), PacketError> {
        self.output.extend_from_slice(&v.to_be_bytes());
        Ok(())
    }

    pub fn serialize_i8(&mut self, v: i8) -> Result<(), PacketError> {
        self.output.extend_from_slice(&v.to_be_bytes());
        Ok(())
    }

    pub fn serialize_i16(&mut self, v: i16) -> Result<(), PacketError> {
        self.output.extend_from_slice(&v.to_be_bytes());
        Ok(())
    }

    pub fn serialize_i32(&mut self, v: i32) -> Result<(), PacketError> {
        self.output.extend_from_slice(&v.to_be_bytes());
        Ok(())
    }

    pub fn serialize_i64(&mut self, v: i64) -> Result<(), PacketError> {
        self.output.extend_from_slice(&v.to_be_bytes());
        Ok(())
    }

    pub fn serialize_string(&mut self, v: String) -> Result<(), PacketError> {
        self.output
            .append(&mut ([(v.len() as u16).to_be_bytes().as_slice(), v.as_bytes()].concat()));
        Ok(())
    }

    pub fn serialize_payload(&mut self, mut v: Vec<u8>) -> Result<(), PacketError> {
        self.output.append(&mut v);
        Ok(())
    }
}

#[derive(Default)]
pub struct PacketDeserializer;

impl PacketDeserializer {
    pub fn deserialize_bool(v: &mut Cursor<&[u8]>) -> Result<(usize, bool), PacketError> {
        if v.remaining() < 1 {
            Err(PacketError::NotEnoughBytes)
        } else {
            Ok((1, v.get_u8() > 0))
        }
    }

    pub fn deserialize_u8(v: &mut Cursor<&[u8]>) -> Result<(usize, u8), PacketError> {
        if v.remaining() < 1 {
            Err(PacketError::NotEnoughBytes)
        } else {
            Ok((1, v.get_u8()))
        }
    }

    pub fn deserialize_u16(v: &mut Cursor<&[u8]>) -> Result<(usize, u16), PacketError> {
        if v.remaining() < 2 {
            Err(PacketError::NotEnoughBytes)
        } else {
            Ok((2, v.get_u16()))
        }
    }

    pub fn deserialize_u32(v: &mut Cursor<&[u8]>) -> Result<(usize, u32), PacketError> {
        if v.remaining() < 4 {
            Err(PacketError::NotEnoughBytes)
        } else {
            Ok((4, v.get_u32()))
        }
    }

    pub fn deserialize_u64(v: &mut Cursor<&[u8]>) -> Result<(usize, u64), PacketError> {
        if v.remaining() < 8 {
            Err(PacketError::NotEnoughBytes)
        } else {
            Ok((8, v.get_u64()))
        }
    }

    pub fn deserialize_f32(v: &mut Cursor<&[u8]>) -> Result<(usize, f32), PacketError> {
        if v.remaining() < 4 {
            Err(PacketError::NotEnoughBytes)
        } else {
            Ok((4, v.get_f32()))
        }
    }

    pub fn deserialize_f64(v: &mut Cursor<&[u8]>) -> Result<(usize, f64), PacketError> {
        if v.remaining() < 8 {
            Err(PacketError::NotEnoughBytes)
        } else {
            Ok((8, v.get_f64()))
        }
    }

    pub fn deserialize_i8(v: &mut Cursor<&[u8]>) -> Result<(usize, i8), PacketError> {
        if v.remaining() < 1 {
            Err(PacketError::NotEnoughBytes)
        } else {
            Ok((1, v.get_i8()))
        }
    }

    pub fn deserialize_i16(v: &mut Cursor<&[u8]>) -> Result<(usize, i16), PacketError> {
        if v.remaining() < 2 {
            Err(PacketError::NotEnoughBytes)
        } else {
            Ok((2, v.get_i16()))
        }
    }

    pub fn deserialize_i32(v: &mut Cursor<&[u8]>) -> Result<(usize, i32), PacketError> {
        if v.remaining() < 4 {
            Err(PacketError::NotEnoughBytes)
        } else {
            Ok((4, v.get_i32()))
        }
    }

    pub fn deserialize_i64(v: &mut Cursor<&[u8]>) -> Result<(usize, i64), PacketError> {
        if v.remaining() < 8 {
            Err(PacketError::NotEnoughBytes)
        } else {
            Ok((8, v.get_i64()))
        }
    }

    pub fn deserialize_string(v: &mut Cursor<&[u8]>) -> Result<(usize, String), PacketError> {
        let len = Self::deserialize_u16(v)?.1 as usize;

        if v.remaining() < len {
            return Err(PacketError::NotEnoughBytes);
        }

        let string = String::from_utf8(v.chunk()[..len].to_vec())
            .map_err(|_e| PacketError::InvalidString)?;
        v.advance(len);
        Ok((len, string))
    }

    pub fn deserialize_payload(v: &mut Cursor<&[u8]>) -> Result<(usize, Vec<u8>), PacketError> {
        let len = Self::deserialize_u16(v)?.1 as usize;

        if v.remaining() < len {
            return Err(PacketError::NotEnoughBytes);
        }

        let vec = v.get_ref()[..len].to_vec();
        v.advance(len);
        Ok((len, vec))
    }
}

pub trait Serialize: Sized {
    fn serialize(&self) -> Result<Vec<u8>, PacketError>;
}

pub trait Deserialize: Sized {
    fn deserialize(bytes: Vec<u8>) -> Result<Self, PacketError> {
        Self::nested_deserialize(&mut std::io::Cursor::new(bytes.as_slice()))
    }

    fn nested_deserialize(cursor: &mut std::io::Cursor<&[u8]>) -> Result<Self, PacketError>;
}
