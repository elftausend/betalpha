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
use packet::{Deserialize, PlayerBlockPlacementPacket};
use procedures::{
    login,
    passive::{player_look, player_position, player_position_and_look},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{
        broadcast::{self, error::TryRecvError},
        mpsc::{self, Sender},
        RwLock,
    },
};
use world::{load_demo::load_entire_world, BlockUpdate, Chunk, World};

// if other clients want to interact with this client
mod global_handlers;
mod movement;
mod packet;
mod utils;
mod world;

// if the server (instantly) reacts to client activity
mod procedures;

use crate::packet::util::*;
use crate::packet::PacketError;

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

    // temp
    let chunks = load_entire_world("./World2/");
    let chunks_lookup = chunks
        .iter()
        .cloned()
        .map(|chunk| ((chunk.chunk_x, chunk.chunk_z), chunk))
        .collect::<std::collections::HashMap<_, _>>();

    let world = Arc::new(RwLock::new(World {
        chunks: chunks_lookup,
    }));

    // let chunks = load_entire_world("/home/elftausend/.minecraft/saves/World1");
    // let chunks = &*Box::leak(chunks.into_boxed_slice());

    let (tx_pos_and_look, rx_pos_and_look) =
        mpsc::channel(256);
    let (tx_pos_and_look_update, _pos_and_look_update_rx) = broadcast::channel(256);

    let (tx_destroy_self_entity, rx_entity_destroy) = mpsc::channel::<i32>(100);
    let (tx_destroy_entities, _) = broadcast::channel(256);

    let (tx_animation, rx_animation) = mpsc::channel::<(i32, Animation)>(100);
    let (tx_broadcast_animations, _) = broadcast::channel::<(i32, Animation)>(100);

    let (tx_block_update, rx_block_updates) = mpsc::channel::<BlockUpdate>(100);
    let (tx_broadcast_block_updates, _) = broadcast::channel::<BlockUpdate>(100);

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
            rx_block_updates,
            tx_broadcast_block_updates: tx_broadcast_block_updates.clone(),
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
            tx_block_update: tx_block_update.clone(),
            rx_global_block_update: tx_broadcast_block_updates.subscribe(),
        };

        let world = world.clone();

        let stream = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let rx_entity_movement = &mut channels.rx_entity_movement;
            let rx_destroy_entities = &mut channels.rx_destroy_entities;
            let rx_global_animations = &mut channels.rx_global_animations;
            let rx_global_block_update = &mut channels.rx_global_block_update;

            // used to clear the prevoius buffered moves ..
            while rx_entity_movement.try_recv().err() != Some(TryRecvError::Empty) {}
            while rx_global_animations.try_recv().err() != Some(TryRecvError::Empty) {}
            while rx_global_block_update.try_recv().err() != Some(TryRecvError::Empty) {}
            while rx_destroy_entities.try_recv().err() != Some(TryRecvError::Empty) {}

            handle_client(stream.0, world, channels).await;
        });
    }
}

pub struct Channels {
    tx_player_pos_and_look: mpsc::Sender<(i32, PositionAndLook, Option<entities::Type>)>,
    rx_entity_movement: broadcast::Receiver<(
        i32,
        Option<entities::Type>,
        PositionAndLook,
        Option<PositionAndLook>,
    )>,
    tx_destroy_self_entity: mpsc::Sender<i32>,
    rx_destroy_entities: broadcast::Receiver<i32>,
    tx_animation: mpsc::Sender<(i32, Animation)>,
    rx_global_animations: broadcast::Receiver<(i32, Animation)>,
    tx_block_update: mpsc::Sender<BlockUpdate>,
    rx_global_block_update: broadcast::Receiver<BlockUpdate>,
}

const SIZE: usize = 1024 * 8;

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
    world: &RwLock<World>,
    state: &RwLock<State>,
    tx_entity: &Sender<(i32, PositionAndLook, Option<entities::Type>)>,
    tx_disconnect: &Sender<i32>,
    tx_animation: &Sender<(i32, Animation)>,
    tx_block_update: &Sender<BlockUpdate>,
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
            1 => login(stream, &mut buf, world, logged_in, state, tx_entity).await?,
            // Handshake
            0x02 => {
                // skip(&mut buf, 1)?;
                let username = get_string(&mut buf)?;
                stream.write_all(&[2, 0, 1, b'-']).await.unwrap();
                stream.flush().await.unwrap();
                println!("ch: {username:?}");
            }
            0x03 => {
                let message = get_string(&mut buf)?;
                println!("{message}")
            }
            0x0A => {
                let _on_ground = get_u8(&mut buf)? != 0;
                // println!("on_ground: {on_ground}");
            }

            0x0B => player_position(&mut buf, state, tx_entity, tx_animation).await?,
            0x0C => player_look(&mut buf, state, tx_entity).await?,
            0x0D => player_position_and_look(&mut buf, state, tx_entity, tx_animation).await?,

            0x12 => {
                let pid = get_i32(&mut buf)?;
                let animation = get_u8(&mut buf)?;
                // println!("animation: {animation}");
                tx_animation
                    .send((pid, Animation::from(animation)))
                    .await
                    .unwrap();
                // println!("{pid} {arm_swinging}")
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

            0x0E => {
                let data = packet::PlayerDiggingPacket::nested_deserialize(&mut buf)?;
                // block broken
                if data.status == 3 {
                    let local_x = if data.x >= 0 {
                        data.x % 16
                    } else {
                        (data.x % 16 + 16) % 16
                    };
                    let local_z = if data.z >= 0 {
                        data.z % 16
                    } else {
                        (data.z % 16 + 16) % 16
                    };

                    // two's complement
                    let (chunk_x, chunk_z) = (data.x >> 4, data.z >> 4);

                    let world = world.read().await;
                    let chunk = world.chunks.get(&(chunk_x, chunk_z)).unwrap();
                    let idx = (data.y as i32 + (local_z * 128 + (local_x * 128 * 16))) as usize;
                    let item_id = chunk.blocks[idx] as i16;

                    tx_block_update
                        .send(BlockUpdate::Break(PlayerBlockPlacementPacket {
                            item_id,
                            x: data.x,
                            y: data.y,
                            z: data.z,
                            face: data.face,
                        }))
                        .await
                        .unwrap();
                }
            }

            0x0F => {
                let data = packet::PlayerBlockPlacementPacket::nested_deserialize(&mut buf)?;
                println!("place: {data:?}");
                if data.item_id != -1 {
                    tx_block_update
                        .send(BlockUpdate::Place(data))
                        .await
                        .unwrap();
                }
            }

            0x10 => {
                let data = packet::HoldingChangePacket::nested_deserialize(&mut buf)?;
            }

            // client inv
            0x05 => {
                let data = packet::PlayerInventoryPacket::nested_deserialize(&mut buf)?;
            }

            0x07 => {
                let data = packet::UseEntityPacket::nested_deserialize(&mut buf)?;
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
    is_crouching: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct PositionAndLook {
    x: f64,
    y: f64,
    z: f64,
    yaw: f32,
    pitch: f32,
}

impl PositionAndLook {
    #[inline]
    pub fn distance(&self, rhs: &PositionAndLook) -> f64 {
        ((self.x - rhs.x).powi(2) + (self.y - rhs.y).powi(2) + (self.z - rhs.z).powi(2)).sqrt()
    }
}

async fn handle_client(stream: TcpStream, world: Arc<RwLock<World>>, channels: Channels) {
    let mut buf = BytesMut::with_capacity(SIZE);

    let Channels {
        tx_player_pos_and_look,
        rx_entity_movement,
        tx_destroy_self_entity,
        rx_destroy_entities,
        tx_animation,
        rx_global_animations: rx_global_animation,
        tx_block_update,
        rx_global_block_update,
    } = channels;

    let stream = Arc::new(RwLock::new(stream));
    let keep_alive_stream = stream.clone();

    let state = Arc::new(RwLock::new(State {
        entity_id: 0,
        username: "".to_string(),
        logged_in: false,
        stance: 0.,
        on_ground: true,
        is_crouching: false,
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

    tokio::task::spawn(global_handlers::block_updates(
        logged_in.clone(),
        rx_global_block_update,
        state.clone(),
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
            &world,
            &state,
            &tx_player_pos_and_look,
            &tx_destroy_self_entity,
            &tx_animation,
            &tx_block_update,
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
