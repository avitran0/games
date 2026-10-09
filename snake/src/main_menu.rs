use api::{Anchor, Button, Color, HEIGHT, Screen, WIDTH, glam::uvec2};

use crate::{game::GameScreen, state::State};

pub struct MainMenu {
    title: api::Sprite,
    last_press: Option<Press>,
}

struct Press {
    tick: usize,
    higher: bool,
}

impl MainMenu {
    pub fn new(ctx: &mut api::ScreenContext<State>) -> Result<Self, api::ScreenAction<State>> {
        let title = ctx.assets.load_sprite(include_bytes!("assets/title.pxs"))?;

        Ok(Self {
            title,
            last_press: None,
        })
    }
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

        if ctx.input.just_pressed(Button::Select) {
            return api::ScreenAction::Quit;
        }

        api::ScreenAction::None
    }

    fn draw(&mut self, state: &State, frame: &mut api::Frame) {
        frame.sprite_anchor(
            &self.title,
            uvec2(WIDTH / 2, HEIGHT / 4).as_ivec2(),
            Anchor::Center,
        );

        frame.text(
            format!("{:?}", state.difficulty),
            uvec2(WIDTH / 2, HEIGHT / 2).as_ivec2(),
            Anchor::Center,
        );

        let up_color = match &self.last_press {
            Some(press) if press.higher => Color::Indigo,
            _ => Color::White,
        };
        frame.text_color(
            '\u{25B3}',
            uvec2(WIDTH / 2, HEIGHT / 2 - 10).as_ivec2(),
            Anchor::Center,
            up_color,
        );

        let down_color = match &self.last_press {
            Some(press) if !press.higher => Color::Indigo,
            _ => Color::White,
        };
        frame.text_color(
            '\u{25BD}',
            uvec2(WIDTH / 2, HEIGHT / 2 + 10).as_ivec2(),
            Anchor::Center,
            down_color,
        );

        frame.text(
            "Press \u{E015} to quit",
            uvec2(WIDTH - 5, HEIGHT - 5).as_ivec2(),
            Anchor::BottomRight,
        );
    }
}
