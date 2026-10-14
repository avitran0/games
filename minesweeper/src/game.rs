use api::{
    Anchor, Button, SpriteOptions,
    glam::{UVec2, ivec2, usizevec2, uvec2},
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

        if ctx.input.just_pressed(Button::A) {
            self.uncover();
        }

        api::ScreenAction::None
    }

    fn draw(&mut self, state: &State, frame: &mut api::Frame) {
        let sprites = self.sprites.set(self.difficulty);

        let stride = self.difficulty.size().x;
        for (pos, tile) in self
            .board
            .iter()
            .enumerate()
            .map(|(index, tile)| (Self::index_to_coord(index, stride as usize), tile))
        {
            let draw_pos = (pos * self.difficulty.tile_size()).as_ivec2();
            if !tile.uncovered {
                frame.sprite(&sprites.covered, draw_pos, SpriteOptions::default());
                continue;
            }

            frame.sprite(&sprites.uncovered, draw_pos, SpriteOptions::default());
            match tile.kind {
                TileKind::Clear => {
                    let neighbors = self.neighbors(pos);
                    frame.text(neighbors.to_string(), draw_pos, Anchor::TopLeft);
                }
                TileKind::Bomb => {
                    frame.sprite(&sprites.bomb, draw_pos, SpriteOptions::default());
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

impl GameScreen {
    fn uncover(&mut self) -> bool {
        let stride = self.difficulty.size().x as usize;
        let index = Self::coord_to_index(self.cursor.pos(), stride);
        let tile = &mut self.board[index];

        if tile.uncovered {
            return true;
        }

        tile.uncovered = true;
        match tile.kind {
            TileKind::Clear => true,
            TileKind::Bomb => false,
        }
    }

    fn generate_board(size: UVec2, first_tile: UVec2) -> Vec<Tile> {
        Vec::new()
    }

    fn neighbors(&self, pos: UVec2) -> usize {
        let size = self.difficulty.size().as_ivec2();
        let mut count = 0;

        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let pos = pos.as_ivec2() + ivec2(dx, dy);
                if pos.x < 0 || pos.y < 0 || pos.x >= size.x || pos.y >= size.y {
                    continue;
                }

                let index = Self::coord_to_index(pos.as_uvec2(), size.x as usize);
                if self.board[index].kind == TileKind::Bomb {
                    count += 1;
                }
            }
        }

        count
    }

    fn coord_to_index(coord: UVec2, stride: usize) -> usize {
        coord.y as usize * stride + coord.x as usize
    }

    fn index_to_coord(index: usize, stride: usize) -> UVec2 {
        usizevec2(index % stride, index / stride).as_uvec2()
    }
}
