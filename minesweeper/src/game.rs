use api::{SpriteOptions, glam::UVec2};

use crate::{difficulty::Difficulty, sprites::Sprites, state::State};

pub struct GameScreen {
    difficulty: Difficulty,
    cursor: UVec2,
    sprites: Sprites,
}

impl GameScreen {
    pub fn new(
        ctx: &mut api::ScreenContext<State>,
        difficulty: Difficulty,
    ) -> Result<Self, api::ScreenAction<State>> {
        let cursor = difficulty.size() / 2;
        let sprites = Sprites::load(ctx)?;

        Ok(Self {
            difficulty,
            cursor,
            sprites,
        })
    }
}

impl api::Screen<State> for GameScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        api::ScreenAction::None
    }

    fn draw(&mut self, state: &State, frame: &mut api::Frame) {
        let cursor_opacity = (frame.tick() as f32 * 0.02).sin().abs() * 0.5 + 0.5;
        let sprites = self.sprites.set(self.difficulty);

        frame.sprite(
            &sprites.cursor,
            (self.cursor * self.difficulty.tile_size()).as_ivec2(),
            SpriteOptions::default()
                .opacity(cursor_opacity)
                .uniform_scale(self.difficulty.tile_scale()),
        );
    }
}
