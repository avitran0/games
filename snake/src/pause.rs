use api::{Anchor, Button, HEIGHT, WIDTH, glam::uvec2};

use crate::{main_menu::MainMenu, state::State};

pub struct PauseScreen;

impl api::Screen<State> for PauseScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        if ctx.input.just_pressed(Button::Start) {
            api::ScreenAction::Pop
        } else if ctx.input.just_pressed(Button::Select) {
            let menu = match MainMenu::new(ctx) {
                Ok(menu) => menu,
                Err(action) => return action,
            };
            api::ScreenAction::ClearAndPush(Box::new(menu))
        } else {
            api::ScreenAction::None
        }
    }

    fn draw(&mut self, _state: &State, frame: &mut api::Frame) {
        frame.text(
            "Paused\nStart to continue\nSelect to quit",
            uvec2(WIDTH / 2, HEIGHT / 2).as_ivec2(),
            Anchor::Center,
        );
    }

    fn is_modal(&self) -> bool {
        true
    }
}
