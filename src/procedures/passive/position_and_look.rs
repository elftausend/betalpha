use std::io::Cursor;

use tokio::sync::{mpsc, RwLock};

use crate::{
    global_handlers::Animation,
    movement::tx_crouching_animation,
    packet::{
        self,
        util::{get_f32, get_u8},
        Deserialize, PacketError,
    },
    PositionAndLook, State,
};

pub async fn player_position(
    buf: &mut Cursor<&[u8]>,
    state: &RwLock<State>,
    tx_entity: &mpsc::Sender<(i32, PositionAndLook, Option<String>)>,
    tx_animation: &mpsc::Sender<(i32, Animation)>,
) -> Result<(), PacketError> {
    let packet::PlayerPositionPacket {
        x,
        y,
        stance,
        z,
        on_ground,
    } = packet::PlayerPositionPacket::nested_deserialize(buf)?;
    let outer_state;
    let eid;
    {
        let mut state = state.write().await;
        state.stance = stance;
        state.on_ground = on_ground;
        state.position_and_look.x = x;
        state.position_and_look.y = y;
        state.position_and_look.z = z;
        outer_state = (state.entity_id, state.position_and_look);
        eid = state.entity_id;
    }

    tx_crouching_animation(eid, stance, y, tx_animation, state)
        .await
        .unwrap();
    tx_entity
        .send((outer_state.0, outer_state.1, None))
        .await
        .unwrap();
    // println!("{x} {y} {stance} {z} {on_ground}");

    Ok(())
}

pub async fn player_look(
    buf: &mut Cursor<&[u8]>,
    state: &RwLock<State>,
    tx_entity: &mpsc::Sender<(i32, PositionAndLook, Option<String>)>,
) -> Result<(), PacketError> {
    let yaw = get_f32(buf)?;
    let pitch = get_f32(buf)?;
    let on_ground = get_u8(buf)? != 0;

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

    Ok(())
}

pub async fn player_position_and_look(
    buf: &mut Cursor<&[u8]>,
    state: &RwLock<State>,
    tx_entity: &mpsc::Sender<(i32, PositionAndLook, Option<String>)>,
    tx_animation: &mpsc::Sender<(i32, Animation)>,
) -> Result<(), PacketError> {
    let packet::PlayerPositionLookPacket {
        x,
        y,
        stance,
        z,
        yaw,
        pitch,
        on_ground,
    } = packet::PlayerPositionLookPacket::nested_deserialize(buf)?;

    let outer_state;
    let eid;
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
        eid = state.entity_id;
    }

    tx_crouching_animation(eid, stance, y, tx_animation, state)
        .await
        .unwrap();
    tx_entity
        .send((outer_state.0, outer_state.1, None))
        .await
        .unwrap();

    // println!("{x} {y} {stance} {z} {yaw} {pitch} {on_ground}");

    Ok(())
}
