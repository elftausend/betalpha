use std::sync::{atomic::AtomicBool, Arc};

use tokio::{
    net::TcpStream,
    sync::{broadcast, RwLock},
};

use crate::{
    packet::{self, util::SendPacket},
    State,
};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Animation {
    None = 0,
    Swing = 1,
    Death = 2,

    Sitting = 100,
    NotSitting = 101,
    Fire = 102,
    NotFire = 103,
    Crouching = 104,
    NotCrouching = 105,
}

impl From<u8> for Animation {
    #[inline]
    fn from(value: u8) -> Self {
        match value {
            0 => Animation::None,
            1 => Animation::Swing,
            2 => Animation::Death,
            100 => Animation::Sitting,
            101 => Animation::NotSitting,
            102 => Animation::Fire,
            103 => Animation::NotFire,
            104 => Animation::Crouching,
            105 => Animation::NotCrouching,

            // error
            _ => Animation::None,
        }
    }
}

pub async fn animations(
    logged_in: Arc<AtomicBool>,
    mut rx_animations: broadcast::Receiver<(i32, Animation)>,
    state: Arc<RwLock<State>>,
    stream: Arc<RwLock<TcpStream>>,
) {
    loop {
        if !logged_in.load(std::sync::atomic::Ordering::Relaxed) {
            continue;
        }
        if let Ok((entity_id, animation)) = rx_animations.recv().await {
            if entity_id == state.read().await.entity_id {
                continue;
            }

            packet::AnimationPacket {
                entity_id,
                animate: animation as u8,
            }
            .send(&mut *stream.write().await)
            .await
            .unwrap()
        }
    }
}
