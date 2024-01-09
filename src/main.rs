use std::{
    future::Future,
    io::Cursor,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        Arc,
    },
};

use bytes::{Buf, BytesMut};

use global_handlers::{collection_center, Animation, CollectionCenter};
use procedures::login;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{
        broadcast::{self, error::TryRecvError},
        mpsc::{self, Sender},
        RwLock,
    },
};
use world::{load_demo::load_entire_world, Chunk};

// if other clients want to interact with this client
mod global_handlers;
mod packet;
mod utils;
mod world;

// if the server (instantly) reacts to client activity
mod procedures;

use crate::packet::PacketError;
use crate::packet::{util::*, Deserialize};

// mod byte_man;
// pub use byte_man::*;

mod entities;

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

    let chunks = load_entire_world("./World2/");
    let chunks = &*Box::leak(chunks.into_boxed_slice());
    let (tx_pos_and_look, rx_pos_and_look) =
        mpsc::channel::<(i32, PositionAndLook, Option<String>)>(256);
    let (tx_pos_and_look_update, _pos_and_look_update_rx) = broadcast::channel(256);

    let (tx_destroy_self_entity, rx_entity_destroy) = mpsc::channel::<i32>(100);
    let (tx_destroy_entities, _) = broadcast::channel(256);

    let (tx_animation, rx_animation) = mpsc::channel::<(i32, Animation)>(100);
    let (tx_broadcast_animations, _) = broadcast::channel::<(i32, Animation)>(100);

    // several maps - avoid cloning of username (remove username from state -> username lookup ?)
    let entity_positions = std::collections::HashMap::new();
    let entity_username = std::collections::HashMap::new();

    tokio::spawn(collection_center(
        entity_username,
        entity_positions,
        CollectionCenter {
            rx_pos_and_look,
            tx_pos_and_look_update: tx_pos_and_look_update.clone(),
            rx_entity_destroy,
            tx_destroy_entities: tx_destroy_entities.clone(),
            rx_animation,
            tx_broadcast_animations: tx_broadcast_animations.clone(),
        },
    ));

    loop {
        let mut channels = Channels {
            tx_player_pos_and_look: tx_pos_and_look.clone(),
            rx_entity_movement: tx_pos_and_look_update.clone().subscribe(),
            tx_destroy_self_entity: tx_destroy_self_entity.clone(),
            rx_destroy_entities: tx_destroy_entities.clone().subscribe(),
            tx_animation: tx_animation.clone(),
            rx_global_animations: tx_broadcast_animations.subscribe(),
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
    tx_animation: mpsc::Sender<(i32, Animation)>,
    rx_global_animations: broadcast::Receiver<(i32, Animation)>,
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

fn get_id() -> i32 {
    static COUNTER: AtomicI32 = AtomicI32::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

// TODO: add checking with peak (faster) [I won't do it]
// TODO: use Arc rwlock
async fn parse_packet(
    stream: &mut TcpStream,
    buf: &BytesMut,
    chunks: &[Chunk],
    state: &RwLock<State>,
    tx_entity: &Sender<(i32, PositionAndLook, Option<String>)>,
    tx_disconnect: &Sender<i32>,
    tx_animation: &Sender<(i32, Animation)>,
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
            1 => login(stream, &mut buf, &chunks, logged_in, state, tx_entity).await?,
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
                let packet::PlayerPositionPacket {
                    x,
                    y,
                    stance,
                    z,
                    on_ground,
                } = packet::PlayerPositionPacket::nested_deserialize(&mut buf)?;

                let outer_state;
                {
                    let mut state = state.write().await;
                    state.stance = stance;
                    state.on_ground = on_ground;
                    state.position_and_look.x = x;
                    state.position_and_look.y = y;
                    state.position_and_look.z = z;
                    outer_state = (state.entity_id, state.position_and_look);
                }
                tx_entity
                    .send((outer_state.0, outer_state.1, None))
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

                    state.on_ground = on_ground;

                    outer_state = (state.entity_id, state.position_and_look);
                }
                tx_entity
                    .send((outer_state.0, outer_state.1, None))
                    .await
                    .unwrap();
                // println!("{yaw} {pitch} {on_ground}");
            }

            0x0D => {
                let packet::PlayerPositionLookPacket {
                    x,
                    y,
                    stance,
                    z,
                    yaw,
                    pitch,
                    on_ground,
                } = packet::PlayerPositionLookPacket::nested_deserialize(&mut buf)?;

                let outer_state;
                {
                    let mut state = state.write().await;
                    state.position_and_look.x = x;
                    state.position_and_look.y = y;
                    state.position_and_look.z = z;
                    state.position_and_look.yaw = yaw;
                    state.position_and_look.pitch = pitch;

                    state.on_ground = on_ground;
                    state.stance = stance;

                    outer_state = (state.entity_id, state.position_and_look);
                }
                tx_entity
                    .send((outer_state.0, outer_state.1, None))
                    .await
                    .unwrap();

                // println!("{x} {y} {stance} {z} {yaw} {pitch} {on_ground}");
            }
            0x12 => {
                let pid = get_i32(&mut buf)?;
                let arm_swinging = get_u8(&mut buf)? > 0;
                if arm_swinging {
                    tx_animation.send((pid, Animation::Swing)).await.unwrap();
                }
                println!("{pid} {arm_swinging}")
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
    stance: f64,
    on_ground: bool,
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
        tx_animation,
        rx_global_animations: rx_global_animation,
    } = channels;

    let stream = Arc::new(RwLock::new(stream));
    let keep_alive_stream = stream.clone();

    let state = Arc::new(RwLock::new(State {
        entity_id: 0,
        username: "".to_string(),
        logged_in: false,
        stance: 0.,
        on_ground: true,
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

    // animations
    tokio::task::spawn(global_handlers::animations(
        logged_in.clone(),
        rx_global_animation,
        state.clone(),
        stream.clone(),
    ));

    // tokio::task::spawn(global_handlers::animations(logged_in.clone(), rx_animations, stream));

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
            &tx_animation,
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
