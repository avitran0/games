use api::{Anchor, Button, HEIGHT, WIDTH, glam::uvec2};

use crate::state::State;

pub struct PauseScreen;

impl api::Screen<State> for PauseScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        if ctx.input.just_pressed(Button::Start) {
            api::ScreenAction::Pop
        } else {
            api::ScreenAction::None
        }
    }

    fn draw(&mut self, _state: &State, frame: &mut api::Frame) {
        frame.text(
            "Paused",
            uvec2(WIDTH / 2, HEIGHT / 2).as_ivec2(),
            Anchor::Center,
        );
    }

    fn is_modal(&self) -> bool {
        true
    }
}
