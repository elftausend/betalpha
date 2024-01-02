#![feature(read_buf)]
use std::{io::{Read, Write, Cursor}, collections::HashMap};

use bytes::{BytesMut, Buf};
use tokio::{net::{TcpListener, TcpStream}, io::{AsyncReadExt, AsyncWriteExt}};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:25565").await.unwrap();

    loop {
        let stream = listener.accept().await.unwrap();
         tokio::spawn(async move {
            handle_client(stream.0).await;
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
    U8
}

pub enum Error {
    Incomplete
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
async fn parse_packet(stream: &mut TcpStream, buf: &BytesMut) -> Result<usize, Error> {
    let mut buf = Cursor::new(&buf[..]);

    let packet_id = get_u8(&mut buf)?;
    println!("packet_id: {packet_id}");

    println!("buf: {buf:?}");

    match packet_id {
        1 => {
            let protocol_version = get_i32(&mut buf)?;
            skip(&mut buf, 1)?;
            let username = get_string(&mut buf)?;
            skip(&mut buf, 1)?;
            let _password = get_string(&mut buf)?;

            let entity_id = 1337i32;
            let seed = 31423422i64;
            let dimension = 0i8; // -1 hell

            let mut packet = vec![1];
            packet.extend_from_slice(&entity_id.to_be_bytes());
            
            packet.extend_from_slice(&[0,0, 0,0]);
            #[rustfmt::skip]
            // packet.extend_from_slice(&[0, 0,0, 0, 0,0, 0]);
            packet.extend_from_slice(&seed.to_be_bytes());
            // packet.extend_from_slice(&[0 ]);
            packet.extend_from_slice(&dimension.to_be_bytes());

            stream.write_all(&packet).await.unwrap();

            println!("protocol_version {protocol_version}");
            println!("username {username}");
        }
        2 => {
            skip(&mut buf, 1)?;
            let username = get_string(&mut buf)?;
            let ch = ClientHandshake {
                username
            };
            stream.write_all(&[2, 0, 1, b'-']).await.unwrap();
            println!("ch: {ch:?}");
        }
        _ => {}
    }

    Ok(buf.position() as usize)
}


async fn handle_client(mut stream: TcpStream) {
    
    let mut buf = BytesMut::with_capacity(SIZE);

    loop {
        if let Ok(n) = parse_packet(&mut stream, &buf).await {
            buf.advance(n);
            buf.clear(); // some fields in packets are ignored
        }
        if stream.read_buf(&mut buf).await.unwrap() == 0 {
            
            println!("break");
            break;
        }
       
    }
}
