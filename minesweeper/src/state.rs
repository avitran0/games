use crate::{camera::CameraRig, leaderboard::Leaderboard};

#[derive(Default)]
pub struct State {
    pub camera: CameraRig,
    pub leaderboard: Leaderboard,
}

api::impl_app_state!(State);
