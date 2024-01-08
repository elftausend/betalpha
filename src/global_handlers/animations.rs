use std::sync::{atomic::AtomicBool, Arc};

use tokio::{
    net::TcpStream,
    sync::{broadcast, RwLock},
};

#[repr(u8)]
pub enum Animation {
    None = 0,
    Swing = 1,
    Death = 2,

    Sitting = 100,
    NotSitting = 101,
    FIRE = 102,
    NotFire = 103,
    Crouching = 104,
    NotCrouching = 105,
}

pub async fn animations(
    logged_in: Arc<AtomicBool>,
    mut rx_animations: broadcast::Receiver<(i32, Animation)>,
    stream: Arc<RwLock<TcpStream>>,
) {
}
