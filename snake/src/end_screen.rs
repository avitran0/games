use api::{
    Anchor, Button, Color, Screen,
    glam::{ivec2, uvec2},
};

use crate::{main_menu::MainMenu, state::State};

#[derive(Clone, Copy)]
pub enum Outcome {
    Win,
    GameOver,
}

pub struct EndScreen {
    outcome: Outcome,
}

impl EndScreen {
    pub fn new(outcome: Outcome) -> Self {
        Self { outcome }
    }
}

impl Screen<State> for EndScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        if ctx.input.just_pressed(Button::Start) || ctx.input.just_pressed(Button::A) {
            let menu = match MainMenu::new(ctx) {
                Ok(menu) => menu,
                Err(action) => return action,
            };
            api::ScreenAction::Replace(Box::new(menu))
        } else {
            api::ScreenAction::None
        }
    }

    fn draw(&mut self, state: &State, frame: &mut api::Frame) {
        frame.rectangle_with_opacity(ivec2(0, 0), frame.resolution().into(), Color::Black, 0.1);
        let heading = match self.outcome {
            Outcome::Win => "You win!",
            Outcome::GameOver => "Game over.",
        };
        frame.text(
            format!(
                "{heading}\nScore: {}\nPress \u{E000} to return to menu.",
                state.score
            ),
            uvec2(frame.width() / 2, frame.height() / 2).as_ivec2(),
            Anchor::Center,
        );
    }
}
