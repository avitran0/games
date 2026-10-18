use crate::state::State;

pub struct GameScreen {
    paddle: api::Sprite,
    ball: api::Sprite,
}

impl GameScreen {
    pub fn new(ctx: &mut api::ScreenContext<State>) -> Result<Self, api::ScreenAction<State>> {
        let paddle = ctx
            .assets
            .load_sprite(include_bytes!("assets/paddle.pxs"))?;
        let ball = ctx.assets.load_sprite(include_bytes!("assets/ball.pxs"))?;

        Ok(Self { paddle, ball })
    }
}

impl api::Screen<State> for GameScreen {
    fn update(&mut self, _ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        api::ScreenAction::None
    }

    fn draw(&mut self, _state: &State, frame: &mut api::Frame) {
        let center_y = frame.height() as i32 / 2;
        frame.sprite(
            &self.paddle,
            api::glam::ivec2(16, center_y),
            api::SpriteOptions::default().anchor(api::Anchor::Center),
        );
        frame.sprite(
            &self.ball,
            api::glam::ivec2(frame.width() as i32 / 2, center_y),
            api::SpriteOptions::default().anchor(api::Anchor::Center),
        );
    }
}
