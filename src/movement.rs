use tokio::sync::{
    mpsc::{self, error::SendError},
    RwLock,
};

use crate::{global_handlers::Animation, State};

pub async fn tx_crouching_animation(
    eid: i32,
    stance: f64,
    y: f64,
    tx_animation: &mpsc::Sender<(i32, Animation)>,
    state: &RwLock<State>, // to enable sending of animation without long lock
) -> Result<(), SendError<(i32, Animation)>> {
    // is stance directly influenced by the server? (first move packet SC)
    let is_crouching = stance - y < 1.59;

    let mut state = state.write().await;

    if is_crouching ^ state.is_crouching {
        let animation = if is_crouching {
            Animation::Crouching
        } else {
            Animation::NotCrouching
        };
        state.is_crouching = is_crouching;
        drop(state);
        tx_animation.send((eid, animation)).await?;
    }

    Ok(())
}
