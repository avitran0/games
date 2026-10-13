use crate::{difficulty::Difficulty, state::State};

#[derive(Default)]
pub struct MainMenu {
    difficulty: Difficulty,
}

impl api::Screen<State> for MainMenu {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        api::ScreenAction::None
    }

    fn draw(&mut self, state: &State, frame: &mut api::Frame) {

    }
}
