use api::glam::UVec2;

use crate::{difficulty::Difficulty, state::State};

pub struct GameScreen {
    cursor: UVec2,
}

impl GameScreen {
    pub fn new(difficulty: Difficulty) -> Self {
        Self {
            cursor: difficulty.size() / 2,
        }
    }
}

impl api::Screen<State> for GameScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        api::ScreenAction::None
    }

    fn draw(&mut self, state: &State, frame: &mut api::Frame) {}
}
