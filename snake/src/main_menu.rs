use api::{Anchor, Button, Color, HEIGHT, Screen, WIDTH, glam::uvec2};

use crate::{game::GameScreen, state::State};

#[derive(Default)]
pub struct MainMenu {
    last_press: Option<Press>,
}

struct Press {
    tick: usize,
    higher: bool,
}

impl Screen<State> for MainMenu {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        if let Some(press) = &self.last_press
            && ctx.tick - press.tick > 10
        {
            self.last_press = None;
        }

        if ctx.input.just_pressed(Button::Up) {
            ctx.state.difficulty.higher();
            self.last_press = Some(Press {
                tick: ctx.tick,
                higher: true,
            });
        }
        if ctx.input.just_pressed(Button::Down) {
            ctx.state.difficulty.lower();
            self.last_press = Some(Press {
                tick: ctx.tick,
                higher: false,
            });
        }

        if ctx.input.just_pressed(Button::Start) {
            return match GameScreen::new(ctx) {
                Ok(game) => api::ScreenAction::Replace(Box::new(game)),
                Err(action) => action,
            };
        }

        api::ScreenAction::None
    }

    fn draw(&mut self, state: &State, frame: &mut api::Frame) {
        frame.text(
            format!("{:?}", state.difficulty),
            uvec2(WIDTH / 2, HEIGHT / 2).as_ivec2(),
            Anchor::Center,
        );

        let up_color = match &self.last_press {
            Some(press) if press.higher => Color::RoyalBlue,
            _ => Color::White,
        };
        frame.text_color(
            '\u{25B3}',
            uvec2(WIDTH / 2, HEIGHT / 2 - 10).as_ivec2(),
            Anchor::Center,
            up_color,
        );

        let down_color = match &self.last_press {
            Some(press) if !press.higher => Color::RoyalBlue,
            _ => Color::White,
        };
        frame.text_color(
            '\u{25BD}',
            uvec2(WIDTH / 2, HEIGHT / 2 + 10).as_ivec2(),
            Anchor::Center,
            down_color,
        );
    }
}
