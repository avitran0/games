use api::{
    Button, SpriteOptions,
    glam::{UVec2, usizevec2},
};

use crate::{
    cursor::Cursor,
    difficulty::Difficulty,
    pause::PauseScreen,
    sprites::Sprites,
    state::State,
    tile::{Tile, TileKind},
};

pub struct GameScreen {
    sprites: Sprites,
    difficulty: Difficulty,
    cursor: Cursor,
    board: Vec<Tile>,
}

impl GameScreen {
    pub fn new(
        ctx: &mut api::ScreenContext<State>,
        difficulty: Difficulty,
    ) -> Result<Self, api::ScreenAction<State>> {
        let cursor = Cursor::new(difficulty);
        let sprites = Sprites::load(ctx)?;
        let board = vec![Tile::default(); difficulty.board_len()];

        Ok(Self {
            sprites,
            difficulty,
            cursor,
            board,
        })
    }
}

impl api::Screen<State> for GameScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        if ctx.input.just_pressed(Button::Start) {
            return api::ScreenAction::Push(Box::new(PauseScreen::new()));
        }

        self.cursor.tick(ctx);

        api::ScreenAction::None
    }

    fn draw(&mut self, state: &State, frame: &mut api::Frame) {
        let sprites = self.sprites.set(self.difficulty);

        let stride = self.difficulty.size().x;
        for (pos, tile) in self
            .board
            .iter()
            .enumerate()
            .map(|(index, tile)| (index_to_coord(index, stride as usize), tile))
        {
            let pos = (pos * self.difficulty.tile_size()).as_ivec2();
            if !tile.uncovered {
                frame.sprite(&sprites.covered, pos, SpriteOptions::default());
                continue;
            }

            match tile.kind {
                TileKind::Clear => {}
                TileKind::Bomb => {
                    frame.sprite(&sprites.bomb, pos, SpriteOptions::default());
                }
            }
        }

        let cursor_opacity = (frame.tick() as f32 * 0.02).sin().abs() * 0.5 + 0.5;
        frame.sprite(
            &sprites.cursor,
            (self.cursor.pos() * self.difficulty.tile_size()).as_ivec2(),
            SpriteOptions::default().opacity(cursor_opacity),
        );
    }
}

fn generate_board(size: UVec2, first_tile: UVec2) -> Vec<Tile> {
    Vec::new()
}

fn index_to_coord(index: usize, stride: usize) -> UVec2 {
    usizevec2(index % stride, index / stride).as_uvec2()
}
