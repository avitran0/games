use api::{
    Anchor, HEIGHT, Screen, SpriteOptions, WIDTH,
    glam::{ivec2, uvec2},
};
use ui::{Alignment, Rect, Ui};

use crate::{
    config::{CONFIG_NAME, Config},
    game::GameScreen,
    snakes::{Snake, SnakeSprites},
    state::State,
};

pub struct MainMenu {
    title: api::Sprite,
    snakes: Vec<SnakeSprites>,
    ui: Ui,
}

impl MainMenu {
    pub fn new(ctx: &mut api::ScreenContext<State>) -> Result<Self, api::ScreenAction<State>> {
        let title = ctx.assets.load_sprite(include_bytes!("assets/title.pxs"))?;
        let snakes = Snake::load_all(ctx)?;
        ctx.state.config = api::load(CONFIG_NAME)?;
        let mut ui = Ui::new();
        ui.set_alignment(Alignment::Center);
        ui.set_bounds(Rect::new(0, 64, WIDTH, HEIGHT - 64));

        Ok(Self { title, snakes, ui })
    }
}

impl Screen<State> for MainMenu {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        self.ui.begin_frame(ctx);

        if self.ui.button("Start").clicked {
            return match GameScreen::new(ctx) {
                Ok(game) => api::ScreenAction::Replace(Box::new(game)),
                Err(action) => action,
            };
        }

        self.ui.combo_box("Difficulty", &mut ctx.state.difficulty);

        if self
            .ui
            .combo_box("Snake", &mut ctx.state.config.snake)
            .changed
            && let Err(action) = api::save::<State, Config>(CONFIG_NAME, &ctx.state.config)
        {
            return action;
        }

        if self.ui.button("Quit").clicked {
            return api::ScreenAction::Quit;
        }

        api::ScreenAction::None
    }

    fn draw(&mut self, state: &State, frame: &mut api::Frame) {
        self.ui.paint(frame);

        frame.sprite(
            &self.title,
            uvec2(WIDTH / 2, HEIGHT / 4).as_ivec2(),
            SpriteOptions::default().anchor(Anchor::Center),
        );

        let snake = &self.snakes[state.config.snake.index()];
        frame.sprite(
            &snake.head,
            ivec2(28, 12),
            SpriteOptions::default()
                .rotation(90.0)
                .rotation_anchor(Anchor::Center),
        );
        frame.sprite(
            &snake.body_angled,
            ivec2(12, 12),
            SpriteOptions::default()
                .rotation(90.0)
                .rotation_anchor(Anchor::Center),
        );
        frame.sprite(&snake.tail, ivec2(12, 28), SpriteOptions::default());
    }
}
