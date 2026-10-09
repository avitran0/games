use api::{Anchor, Button, HEIGHT, WIDTH, glam::uvec2};

use crate::{game::GameScreen, state::State};

pub struct MainMenu;

impl api::Screen<State> for MainMenu {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        if ctx.input.just_pressed(Button::Start) {
            let game = match GameScreen::new(ctx) {
                Ok(game) => game,
                Err(action) => return action,
            };
            api::ScreenAction::Replace(Box::new(game))
        } else if ctx.input.just_pressed(Button::Select) {
            api::ScreenAction::Quit
        } else {
            api::ScreenAction::None
        }
    }

    fn draw(&mut self, _state: &State, frame: &mut api::Frame) {
        frame.text(
            "Pong",
            uvec2(WIDTH / 2, HEIGHT / 2).as_ivec2(),
            Anchor::Center,
        );
    }
}
