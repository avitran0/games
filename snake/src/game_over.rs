use api::{Anchor, Button, HEIGHT, Screen, WIDTH, glam::uvec2};

use crate::{main_menu::MainMenu, state::State};

pub struct GameOverScreen;

impl Screen<State> for GameOverScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        if ctx.input.just_pressed(Button::Start) || ctx.input.just_pressed(Button::A) {
            ctx.state.score = 0;
            api::ScreenAction::Replace(Box::new(MainMenu::default()))
        } else {
            api::ScreenAction::None
        }
    }

    fn draw(&mut self, _state: &State, frame: &mut api::Frame) {
        frame.text(
            "Game over.\nPress \u{E000} to try again.",
            uvec2(WIDTH / 2, HEIGHT / 2).as_ivec2(),
            Anchor::Center,
        );
    }
}
