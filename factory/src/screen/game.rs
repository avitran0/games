use api::glam::uvec2;

use crate::{core::world::World, state::State};

pub struct GameScreen {
    world: World,
}

impl GameScreen {
    pub fn new() -> Self {
        let world = World::new(uvec2(128, 128));

        Self { world }
    }
}

impl api::Screen<State> for GameScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        api::ScreenAction::None
    }

    fn draw(&mut self, state: &State, frame: &mut api::Frame) {

    }
}
