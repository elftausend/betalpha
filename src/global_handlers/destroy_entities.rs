use std::sync::Arc;

use tokio::{
    net::TcpStream,
    sync::{broadcast, RwLock},
};

use crate::packet::{self, util::SendPacket};

pub async fn destroy_entities(
    mut rx_destroy_entities: broadcast::Receiver<i32>,
    entity_destroy_stream: Arc<RwLock<TcpStream>>,
) {
    loop {
        if let Ok(entity_id) = rx_destroy_entities.recv().await {
            packet::DestroyEntityPacket { entity_id }
                .send(&mut *entity_destroy_stream.write().await)
                .await
                .unwrap_or_default();
        }
    }
}
