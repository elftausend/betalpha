use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    sync::{broadcast, RwLock},
};

use crate::{
    entities::{self, spawned_named_entity, Type},
    utils::look_to_i8_range,
    PositionAndLook, State,
};

pub async fn spawn_entities(
    logged_in: Arc<AtomicBool>,
    state_pos_update: Arc<RwLock<State>>,
    mut rx_entity_movement: broadcast::Receiver<(
        i32,
        Type,
        PositionAndLook,
        Option<PositionAndLook>,
    )>,
    pos_update_stream: Arc<RwLock<TcpStream>>,
) {
    let mut seen_before = HashSet::new();
    loop {
        // single core servers
        if !logged_in.load(Ordering::Relaxed) {
            continue;
        }
        let Ok((eid, ty, now, prev)) = rx_entity_movement.recv().await else {
            continue;
        };

        if eid == state_pos_update.read().await.entity_id {
            continue;
        }

        // TODO: add eid is in reach check, unload/destroy entity
        // FIXME: could potentially receive a lot of data / entity information that is intantly discarded

        // println!(
        //     "i am: {}, moved: {eid} {now:?}, prev: {prev:?}",
        //     state_pos_update.read().await.entity_id
        // );

        if !seen_before.contains(&eid) {
            let mut pos_update_stream = pos_update_stream.write().await;

            match ty {
                entities::Type::Player(name) => {
                    spawned_named_entity(&mut pos_update_stream, eid, &name, &now)
                        .await
                        .unwrap()
                }
            };

            let mut entity_spawn = vec![0x1E];
            entity_spawn.extend_from_slice(&eid.to_be_bytes());

            pos_update_stream.write_all(&entity_spawn).await.unwrap();
            pos_update_stream.flush().await.unwrap();
        }

        seen_before.insert(eid);

        if let Some(prev) = prev {
            // check if travelled blocks is > 4 (teleport)

            let x = ((now.x - prev.x) * 32.).round() as i8;
            let y = ((now.y - prev.y) * 32.).round() as i8;
            let z = ((now.z - prev.z) * 32.).round() as i8;

            let (yaw, pitch) = look_to_i8_range(now.yaw, now.pitch);

            // println!("yaw: {yawf} {} pitch: {pitch}", yaw);

            let mut entity_look_and_move = vec![0x21];
            entity_look_and_move.extend_from_slice(&eid.to_be_bytes());
            entity_look_and_move.extend_from_slice(&[
                x.to_be_bytes()[0],
                y.to_be_bytes()[0],
                z.to_be_bytes()[0],
                yaw.to_be_bytes()[0],
                pitch.to_be_bytes()[0],
            ]);

            let mut pos_update_stream = pos_update_stream.write().await;
            pos_update_stream
                .write_all(&entity_look_and_move)
                .await
                .unwrap();
            pos_update_stream.flush().await.unwrap();
        }
    }
}
