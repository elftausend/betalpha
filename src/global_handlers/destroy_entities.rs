use std::sync::Arc;

use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    sync::{broadcast, RwLock},
};

pub async fn destroy_entities(
    mut rx_destroy_entities: broadcast::Receiver<i32>,
    entity_destroy_stream: Arc<RwLock<TcpStream>>,
) {
    loop {
        if let Ok(eid) = rx_destroy_entities.recv().await {
            let mut destroy_entity = vec![0x1D];
            destroy_entity.extend_from_slice(&eid.to_be_bytes());

            let mut destroy_entity_stream = entity_destroy_stream.write().await;
            destroy_entity_stream
                .write_all(&destroy_entity)
                .await
                .unwrap();
            destroy_entity_stream.flush().await.unwrap();
        }
    }
}
