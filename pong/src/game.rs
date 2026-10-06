use crate::state::State;

pub struct GameScreen {

}

impl GameScreen {
    const PADDLE_HEIGHT: u32 = 24;
}

impl api::Screen<State> for GameScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        api::ScreenAction::None
    }

    fn draw(&mut self, state: &State, frame: &mut api::Frame) {

    }
}
