use api::{
    Anchor, Button, Rng, SpriteOptions,
    glam::{UVec2, ivec2, usizevec2, uvec2},
};

use crate::{
    cursor::Cursor,
    pause::PauseScreen,
    sprites::Sprites,
    state::State,
    tile::{Tile, TileKind},
};

pub const TILE_SIZE: u32 = 16;

pub struct GameScreen {
    sprites: Sprites,
    size: UVec2,
    cursor: Cursor,
    board: Vec<Tile>,
    generated: bool,
}

impl GameScreen {
    pub const MIN_BOARD_SIZE: u32 = 5;
    pub const MAX_BOARD_WIDTH: u32 = api::WIDTH / TILE_SIZE;
    pub const MAX_BOARD_HEIGHT: u32 = api::HEIGHT / TILE_SIZE;

    pub fn new(
        ctx: &mut api::ScreenContext<State>,
        size: UVec2,
    ) -> Result<Self, api::ScreenAction<State>> {
        // Keep the full board visible at the fixed tile size.
        let size = uvec2(
            size.x.clamp(Self::MIN_BOARD_SIZE, Self::MAX_BOARD_WIDTH),
            size.y.clamp(Self::MIN_BOARD_SIZE, Self::MAX_BOARD_HEIGHT),
        );
        let cursor = Cursor::new(size);
        let sprites = Sprites::load(ctx)?;
        let board = vec![Tile::default(); size.x as usize * size.y as usize];

        Ok(Self {
            sprites,
            size,
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

    fn draw(&mut self, _state: &State, frame: &mut api::Frame) {
        let stride = self.size.x;

        for (pos, tile) in self
            .board
            .iter()
            .enumerate()
            .map(|(index, tile)| (Self::index_to_coord(index, stride as usize), tile))
        {
            let draw_pos = (pos * TILE_SIZE).as_ivec2();
            if !tile.uncovered {
                frame.sprite(&self.sprites.covered, draw_pos, SpriteOptions::default());
                if tile.flagged {
                    frame.sprite(&self.sprites.flag, draw_pos, SpriteOptions::default());
                }
                continue;
            }

            frame.sprite(&self.sprites.uncovered, draw_pos, SpriteOptions::default());
            match tile.kind {
                TileKind::Clear => {
                    let neighbors = self.neighbors(pos);
                    frame.text(neighbors.to_string(), draw_pos, Anchor::TopLeft);
                }
                TileKind::Bomb => {
                    frame.sprite(&self.sprites.bomb, draw_pos, SpriteOptions::default());
                }
            }
        }

        let cursor_opacity = (frame.tick() as f32 * 0.02).sin().abs() * 0.5 + 0.5;
        frame.sprite(
            &self.sprites.cursor,
            (self.cursor.pos() * TILE_SIZE).as_ivec2(),
            SpriteOptions::default().opacity(cursor_opacity),
        );
    }
}

impl GameScreen {
    fn uncover(&mut self) -> bool {
        let start = self.cursor.pos();
        if !self.generated {
            self.board = Self::generate_board(self.size, start);
            self.generated = true;
        }

        let stride = self.size.x as usize;
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

            // Numbered tiles should be revealed, but don't propagate through them.
            if self.neighbors(pos) != 0 {
                continue;
            }

            // Zero-neighbor tiles propagate to each valid adjacent position.
            let size = self.size.as_ivec2();
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
        let stride = self.size.x as usize;
        let index = Self::coord_to_index(self.cursor.pos(), stride);
        self.board[index].flagged = !self.board[index].flagged;
    }

    fn generate_board(size: UVec2, first_tile: UVec2) -> Vec<Tile> {
        let mut board = vec![Tile::default(); size.x as usize * size.y as usize];

        let stride = size.x as usize;
        let mut candidates: Vec<usize> = (0..board.len())
            .filter(|&index| {
                let pos = Self::index_to_coord(index, stride);
                pos.x.abs_diff(first_tile.x) > 1 || pos.y.abs_diff(first_tile.y) > 1
            })
            .collect();

        let bomb_count = (size.x * size.y / 10) as usize;
        let mut rng = Rng::new();
        rng.shuffle(&mut candidates);
        for index in candidates.into_iter().take(bomb_count) {
            board[index].kind = TileKind::Bomb;
        }

        board
    }

    fn neighbors(&self, pos: UVec2) -> usize {
        let size = self.size.as_ivec2();
        let mut count = 0;

        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let neighbor = pos.as_ivec2() + ivec2(dx, dy);
                if neighbor.x < 0 || neighbor.y < 0 || neighbor.x >= size.x || neighbor.y >= size.y
                {
                    continue;
                }

                let index = Self::coord_to_index(neighbor.as_uvec2(), size.x as usize);
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
