mod spawn_entity;
pub use spawn_entity::*;

mod destroy_entities;
pub use destroy_entities::*;

mod animations;
pub use animations::*;

mod blocks;
pub use blocks::*;

use tokio::sync::{broadcast, mpsc};

use crate::{
    entities, get_id, packet::{PlayerBlockPlacementPacket, Item}, world::BlockUpdate, PositionAndLook,
};
use std::collections::HashMap;

pub struct CollectionCenter {
    pub rx_pos_and_look: mpsc::Receiver<(i32, PositionAndLook, Option<entities::Type>)>,
    pub tx_pos_and_look_update: broadcast::Sender<(
        i32,
        Option<entities::Type>,
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
    mut entity_type: HashMap<i32, entities::Type>,
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
        if let Ok((eid, pos_and_look, entity)) = rx_pos_and_look.try_recv() {
            let prev_pos_and_look = entity_positions.insert(eid, pos_and_look);
            if let Some(ty) = entity {
                entity_type.insert(eid, ty);
            }

            // if a player logs in (prev pos is none), not moving entities should be sent
            if prev_pos_and_look.is_none() {
                for (eid, pos_and_look) in &entity_positions {
                    tx_pos_and_look_update
                        .send((
                            *eid,
                            Some(entity_type[eid].clone()),
                            *pos_and_look,
                            None,
                        ))
                        .unwrap();
                }
            }

            tx_pos_and_look_update
                .send((
                    eid,
                    Some(entity_type[&eid].clone()), // could be none.. 
                    // None,
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
                BlockUpdate::Break(block_info) => {
                    let eid = get_id();
                    let pos = PositionAndLook {
                        x: block_info.x as f64,
                        y: block_info.y as f64,
                        z: block_info.z as f64,
                        yaw: 0.,
                        pitch: 0.,
                    };
                    entity_positions.insert(eid, pos);
                    let item = entities::Type::Item(Item { item_id: block_info.item_id, count: 1, uses: 0});
                    entity_type.insert(eid, item.clone());
                    tx_pos_and_look_update
                        .send((eid, Some(item), pos, None))
                        .unwrap();
                }
            }
            tx_broadcast_block_updates.send(block_update).unwrap();
        }

        // destroy entities
        if let Ok(eid) = rx_entity_destroy.try_recv() {
            entity_positions.remove(&eid);
            entity_type.remove(&eid);

            tx_destroy_entities.send(eid).unwrap();
        }

        // maybe use another container for items
        for (eid, ty) in entity_type.iter() {
            let entities::Type::Item(item) = ty else {
                continue
            };
            let pos = entity_positions[eid];
            for (check_collect_eid, check_pos) in &entity_positions {
                if eid == check_collect_eid {
                    continue;
                }
                let entities::Type::Player(username) = entity_type[check_collect_eid].clone() else {
                    continue
                };
                // https://www.spigotmc.org/threads/item-pickup-radius.337271/
                if pos.distance(check_pos) < 2.04 {
                    println!("{username} can collect {item:?}",);
                }
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs_f64(0.0001)).await;
    }
}
