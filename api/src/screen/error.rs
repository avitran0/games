use std::error::Error;

use glam::uvec2;

use crate::{Anchor, Button, Frame, HEIGHT, Screen, ScreenAction, ScreenContext, WIDTH};

pub struct ErrorScreen<E: Error + 'static> {
    error: E,
}

impl<E: Error> ErrorScreen<E> {
    pub(crate) fn new(error: E) -> Self {
        Self { error }
    }
}

impl<E: Error, State> Screen<State> for ErrorScreen<E> {
    fn update(&mut self, ctx: &mut ScreenContext<'_, State>) -> ScreenAction<State> {
        if ctx.input.just_pressed(Button::A) {
            ScreenAction::Pop
        } else {
            ScreenAction::None
        }
    }

    fn draw(&mut self, _state: &State, frame: &mut Frame) {
        frame.text(
            format!("Error: {}", self.error),
            uvec2(WIDTH / 2, HEIGHT / 2).as_ivec2(),
            Anchor::Center,
        );
        frame.text(
            "Press \u{E000} to continue",
            uvec2(WIDTH, HEIGHT).as_ivec2(),
            Anchor::BottomRight,
        );
    }
}
