use api::{
    Anchor, Button, Rng, SpriteOptions,
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
    generated: bool,
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
            generated: false,
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

        if ctx.input.just_pressed(Button::B) {
            self.flag();
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
                if tile.flagged {
                    frame.sprite(&sprites.flag, draw_pos, SpriteOptions::default());
                }
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
        let start = self.cursor.pos();
        if !self.generated {
            self.board = Self::generate_board(self.difficulty.size(), start);
            self.generated = true;
        }

        let size = self.difficulty.size();
        let stride = size.x as usize;
        let start_index = Self::coord_to_index(start, stride);

        if self.board[start_index].uncovered {
            return true;
        }

        if self.board[start_index].kind == TileKind::Bomb {
            self.board[start_index].uncovered = true;
            return false;
        }

        let mut pending = vec![start];

        while let Some(pos) = pending.pop() {
            let index = Self::coord_to_index(pos, stride);

            if self.board[index].uncovered || self.board[index].kind == TileKind::Bomb {
                continue;
            }

            self.board[index].uncovered = true;

            // numbered tiles should be revealed, but don't propagate
            if self.neighbors(pos) != 0 {
                continue;
            }

            // only tiles with no neighbors propagate
            let size = size.as_ivec2();
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }

                    let adjacent = pos.as_ivec2() + ivec2(dx, dy);
                    if adjacent.x < 0
                        || adjacent.y < 0
                        || adjacent.x >= size.x
                        || adjacent.y >= size.y
                    {
                        continue;
                    }

                    pending.push(adjacent.as_uvec2());
                }
            }
        }

        true
    }

    fn flag(&mut self) {
        let stride = self.difficulty.size().x as usize;
        let index = Self::coord_to_index(self.cursor.pos(), stride);
        self.board[index].flagged = !self.board[index].flagged;
    }

    fn generate_board(size: UVec2, first_tile: UVec2) -> Vec<Tile> {
        let mut board = vec![Tile::default(); size.x as usize * size.y as usize];

        let stride = size.x as usize;
        let mut candidates: Vec<usize> = (0..board.len())
            .filter(|&index| {
                let pos = Self::index_to_coord(index, stride);
                pos.x.abs_diff(first_tile.x) > 1 && pos.y.abs_diff(first_tile.y) > 1
            })
            .collect();

        let bomb_count = size.x * size.y / 10;
        let mut rng = Rng::new();
        rng.shuffle(&mut candidates);
        for tile in candidates.into_iter().take(bomb_count as usize) {
            board[tile].kind = TileKind::Bomb;
        }

        board
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
