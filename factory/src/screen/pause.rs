use crate::state::State;

pub struct PauseMenu {

}

impl api::Screen<State> for PauseMenu {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        api::ScreenAction::None
    }

    fn draw(&mut self, state: &State, frame: &mut api::Frame) {

    }
}
