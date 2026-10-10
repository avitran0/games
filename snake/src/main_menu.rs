use api::{
    Anchor, Button, Color, HEIGHT, Screen, WIDTH,
    glam::{ivec2, uvec2},
};

use crate::{
    config::{CONFIG_NAME, Config},
    game::GameScreen,
    snakes::Snakes,
    state::State,
};

pub struct MainMenu {
    title: api::Sprite,
    snakes: Snakes,
    last_press: Option<Press>,
}

struct Press {
    tick: usize,
    higher: bool,
}

impl MainMenu {
    pub fn new(ctx: &mut api::ScreenContext<State>) -> Result<Self, api::ScreenAction<State>> {
        let title = ctx.assets.load_sprite(include_bytes!("assets/title.pxs"))?;
        let snakes = Snakes::load(ctx)?;
        ctx.state.config = api::load(CONFIG_NAME)?;

        Ok(Self {
            title,
            snakes,
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

        let count = self.snakes.len();
        ctx.state.config.snake %= count;

        if ctx.input.just_pressed(Button::L) {
            ctx.state.config.snake = (ctx.state.config.snake + count - 1) % count;
            if let Err(action) = api::save::<State, Config>(CONFIG_NAME, &ctx.state.config) {
                return action;
            }
        }

        if ctx.input.just_pressed(Button::R) {
            ctx.state.config.snake = (ctx.state.config.snake + 1) % count;
            if let Err(action) = api::save::<State, Config>(CONFIG_NAME, &ctx.state.config) {
                return action;
            }
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

        let snake = self.snakes.get(state.config.snake);
        frame.sprite_rotate(&snake.head, ivec2(28, 12), 90.0, Anchor::Center);
        frame.sprite_rotate(&snake.body_angled, ivec2(12, 12), 90.0, Anchor::Center);
        frame.sprite(&snake.tail, ivec2(12, 28));
    }
}
