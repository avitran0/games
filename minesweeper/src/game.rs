use api::{
    Anchor, Button, Color, Rng, SpriteOptions,
    glam::{UVec2, ivec2, usizevec2, uvec2},
};

use crate::{
    cursor::Cursor,
    pause::PauseScreen,
    sprites::Sprites,
    state::State,
    tile::{Tile, TileKind},
};

pub(crate) const TILE_SIZE: u32 = 16;

pub struct GameScreen {
    sprites: Sprites,
    size: UVec2,
    cursor: Cursor,
    board: Vec<Tile>,
    generated: bool,
}

impl GameScreen {
    pub fn new(
        ctx: &mut api::ScreenContext<State>,
        size: UVec2,
    ) -> Result<Self, api::ScreenAction<State>> {
        let size = size.max(UVec2::ONE);
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
        let sprites = &self.sprites;
        let camera = self.cursor.camera();
        let top_left = camera.screen_to_world(ivec2(0, 0));
        let bottom_right = camera.screen_to_world(ivec2(api::WIDTH as i32, api::HEIGHT as i32));
        let tile_size = TILE_SIZE as f32;
        let max_x = self.size.x as i64 - 1;
        let max_y = self.size.y as i64 - 1;
        let min_x = ((top_left.x / tile_size).floor() as i64 - 1).clamp(0, max_x) as u32;
        let min_y = ((top_left.y / tile_size).floor() as i64 - 1).clamp(0, max_y) as u32;
        let end_x = ((bottom_right.x / tile_size).floor() as i64 + 1).clamp(0, max_x) as u32;
        let end_y = ((bottom_right.y / tile_size).floor() as i64 + 1).clamp(0, max_y) as u32;

        {
            let mut world = frame.world(camera);
            for y in min_y..=end_y {
                for x in min_x..=end_x {
                    self.draw_tile(&mut world, uvec2(x, y));
                }
            }
        }

        let cursor_screen_pos = ivec2(
            (api::WIDTH as i32 - TILE_SIZE as i32) / 2,
            (api::HEIGHT as i32 - TILE_SIZE as i32) / 2,
        );
        let cursor_opacity = (frame.tick() as f32 * 0.02).sin().abs() * 0.5 + 0.5;
        frame.sprite(
            &sprites.cursor,
            cursor_screen_pos,
            SpriteOptions::default().opacity(cursor_opacity),
        );
    }
}

impl GameScreen {
    fn draw_tile(&self, world: &mut api::WorldFrame<'_, '_>, pos: UVec2) {
        let index = Self::coord_to_index(pos, self.size.x as usize);
        let tile = self.board[index];
        let draw_pos = (pos * TILE_SIZE).as_ivec2();

        if !tile.uncovered {
            world.sprite(&self.sprites.covered, draw_pos, SpriteOptions::default());
            if tile.flagged {
                world.sprite(&self.sprites.flag, draw_pos, SpriteOptions::default());
            }
            return;
        }

        world.sprite(&self.sprites.uncovered, draw_pos, SpriteOptions::default());
        match tile.kind {
            TileKind::Clear => {
                let neighbors = self.neighbors(pos);
                if neighbors != 0 {
                    let text_pos = draw_pos + TILE_SIZE as i32 / 2;
                    let color = match neighbors {
                        1 => Color::Indigo,
                        2 => Color::ForestGreen,
                        3 => Color::TallPoppy,
                        4 => Color::PurpleHeart,
                        _ => Color::Bunker,
                    };
                    world.text_color(
                        neighbors.to_string(),
                        text_pos,
                        Anchor::Center,
                        color,
                    );
                }
            }
            TileKind::Bomb => {
                world.sprite(&self.sprites.bomb, draw_pos, SpriteOptions::default());
            }
        }
    }

    fn uncover(&mut self) -> bool {
        let start = self.cursor.pos();
        let start_index = Self::coord_to_index(start, self.size.x as usize);

        if self.board[start_index].uncovered || self.board[start_index].flagged {
            return true;
        }

        if !self.generated {
            Self::generate_board(&mut self.board, self.size, start);
            self.generated = true;
        }

        if self.board[start_index].kind == TileKind::Bomb {
            self.board[start_index].uncovered = true;
            return false;
        }

        let mut pending = vec![start];
        while let Some(pos) = pending.pop() {
            let index = Self::coord_to_index(pos, self.size.x as usize);
            let tile = self.board[index];
            if tile.uncovered || tile.flagged || tile.kind == TileKind::Bomb {
                continue;
            }

            self.board[index].uncovered = true;
            if self.neighbors(pos) == 0 {
                pending.extend(Self::adjacent_positions(pos, self.size));
            }
        }

        true
    }

    fn flag(&mut self) {
        let index = Self::coord_to_index(self.cursor.pos(), self.size.x as usize);
        let tile = &mut self.board[index];
        if !tile.uncovered {
            tile.flagged = !tile.flagged;
        }
    }

    fn generate_board(board: &mut [Tile], size: UVec2, first_tile: UVec2) {
        let stride = size.x as usize;
        let mut candidates: Vec<usize> = (0..board.len())
            .filter(|&index| {
                let pos = Self::index_to_coord(index, stride);
                pos.x.abs_diff(first_tile.x) > 1 || pos.y.abs_diff(first_tile.y) > 1
            })
            .collect();

        let bomb_count = (size.x as u64 * size.y as u64 / 10) as usize;
        let mut rng = Rng::new();
        rng.shuffle(&mut candidates);
        for index in candidates.into_iter().take(bomb_count) {
            board[index].kind = TileKind::Bomb;
        }
    }

    fn neighbors(&self, pos: UVec2) -> usize {
        Self::adjacent_positions(pos, self.size)
            .into_iter()
            .filter(|&neighbor| {
                let index = Self::coord_to_index(neighbor, self.size.x as usize);
                self.board[index].kind == TileKind::Bomb
            })
            .count()
    }

    fn adjacent_positions(pos: UVec2, size: UVec2) -> Vec<UVec2> {
        let mut adjacent = Vec::with_capacity(8);
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let x = pos.x as i32 + dx;
                let y = pos.y as i32 + dy;
                if x >= 0 && y >= 0 && x < size.x as i32 && y < size.y as i32 {
                    adjacent.push(uvec2(x as u32, y as u32));
                }
            }
        }
        adjacent
    }

    fn coord_to_index(coord: UVec2, stride: usize) -> usize {
        coord.y as usize * stride + coord.x as usize
    }

    fn index_to_coord(index: usize, stride: usize) -> UVec2 {
        usizevec2(index % stride, index / stride).as_uvec2()
    }
}
