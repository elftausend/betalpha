mod spawn_entity;
pub use spawn_entity::*;

mod destroy_entities;
pub use destroy_entities::*;

mod animations;
pub use animations::*;

mod blocks;
pub use blocks::*;

use tokio::sync::{broadcast, mpsc};

use crate::{entities, packet::PlayerBlockPlacementPacket, world::BlockUpdate, PositionAndLook};
use std::collections::HashMap;

pub struct CollectionCenter {
    pub rx_pos_and_look: mpsc::Receiver<(i32, PositionAndLook, Option<String>)>,
    pub tx_pos_and_look_update: broadcast::Sender<(
        i32,
        entities::Type,
        PositionAndLook,
        Option<PositionAndLook>,
    )>,
    pub rx_entity_destroy: mpsc::Receiver<i32>,
    pub tx_destroy_entities: broadcast::Sender<i32>,
    pub rx_animation: mpsc::Receiver<(i32, Animation)>,
    pub tx_broadcast_animations: broadcast::Sender<(i32, Animation)>,
    pub rx_block_updates: mpsc::Receiver<BlockUpdate>,
    pub tx_broadcast_block_updates: broadcast::Sender<BlockUpdate>,
}

pub async fn collection_center(
    mut entity_username: HashMap<i32, String>,
    mut entity_positions: HashMap<i32, PositionAndLook>,
    collection_center: CollectionCenter,
) {
    let CollectionCenter {
        mut rx_pos_and_look,
        tx_pos_and_look_update,
        mut rx_entity_destroy,
        tx_destroy_entities,
        mut rx_animation,
        tx_broadcast_animations,
        mut rx_block_updates,
        tx_broadcast_block_updates,
    } = collection_center;

    loop {
        // receive position updates, log in (username)
        if let Ok((eid, pos_and_look, username)) = rx_pos_and_look.try_recv() {
            let prev_pos_and_look = entity_positions.insert(eid, pos_and_look);
            if let Some(username) = username {
                entity_username.insert(eid, username);
            }

            // if a player logs in (prev pos is none), not moving entities should be sent
            if prev_pos_and_look.is_none() {
                for (eid, pos_and_look) in &entity_positions {
                    tx_pos_and_look_update
                        .send((
                            *eid,
                            entities::Type::Player(entity_username[eid].clone()),
                            *pos_and_look,
                            None,
                        ))
                        .unwrap();
                }
            }

            tx_pos_and_look_update
                .send((
                    eid,
                    entities::Type::Player(entity_username[&eid].clone()),
                    pos_and_look,
                    prev_pos_and_look,
                ))
                .unwrap();
        }

        if let Ok((eid, animation)) = rx_animation.try_recv() {
            tx_broadcast_animations.send((eid, animation)).unwrap();
        }

        if let Ok(block_update) = rx_block_updates.try_recv() {
            match block_update {
                BlockUpdate::Place(_) => {}
                BlockUpdate::Break(_) => {}
            }
            tx_broadcast_block_updates.send(block_update).unwrap();
        }

        if let Ok(eid) = rx_entity_destroy.try_recv() {
            entity_positions.remove(&eid);
            entity_username.remove(&eid);

            tx_destroy_entities.send(eid).unwrap();
        }
        tokio::time::sleep(std::time::Duration::from_secs_f64(0.0001)).await;
    }
}
