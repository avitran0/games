use api::{
    Anchor, Color, Rng, SpriteOptions,
    glam::{UVec2, ivec2, usizevec2, uvec2},
};

use crate::{
    difficulty::Difficulty,
    game::TILE_SIZE,
    sprites::Sprites,
    tile::{Tile, TileKind},
};

#[derive(Clone)]
pub struct Board {
    size: UVec2,
    difficulty: Difficulty,
    tiles: Vec<Tile>,
    generated: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UncoverResult {
    Safe,
    HitBomb,
}

impl Board {
    pub fn new(size: UVec2, difficulty: Difficulty) -> Self {
        let size = size.max(UVec2::ONE);
        Self {
            size,
            difficulty,
            tiles: vec![Tile::default(); size.x as usize * size.y as usize],
            generated: false,
        }
    }

    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub fn is_generated(&self) -> bool {
        self.generated
    }

    pub fn difficulty(&self) -> Difficulty {
        self.difficulty
    }

    pub fn tile(&self, pos: UVec2) -> Tile {
        self.tiles[Self::coord_to_index(pos, self.size.x as usize)]
    }

    pub fn uncover(&mut self, start: UVec2) -> UncoverResult {
        let start_index = Self::coord_to_index(start, self.size.x as usize);
        let start_tile = self.tiles[start_index];
        if start_tile.uncovered {
            return self.chord(start);
        }
        if start_tile.flagged {
            return UncoverResult::Safe;
        }

        if !self.generated {
            self.generate(start);
            self.generated = true;
        }

        if self.tiles[start_index].kind == TileKind::Bomb {
            self.tiles[start_index].uncovered = true;
            return UncoverResult::HitBomb;
        }

        self.tiles[start_index].uncovered = true;
        let mut pending = vec![start];
        while let Some(pos) = pending.pop() {
            if self.neighbors(pos) != 0 {
                continue;
            }

            Self::for_each_adjacent(pos, self.size, |neighbor| {
                let index = Self::coord_to_index(neighbor, self.size.x as usize);
                let tile = &mut self.tiles[index];
                if !tile.uncovered && !tile.flagged && tile.kind != TileKind::Bomb {
                    tile.uncovered = true;
                    pending.push(neighbor);
                }
            });
        }

        UncoverResult::Safe
    }

    fn chord(&mut self, pos: UVec2) -> UncoverResult {
        let tile = self.tile(pos);
        if !tile.uncovered || tile.kind != TileKind::Clear {
            return UncoverResult::Safe;
        }

        let mut adjacent_flags = 0;
        let mut adjacent_tiles = Vec::new();
        Self::for_each_adjacent(pos, self.size, |neighbor| {
            if self.tile(neighbor).flagged {
                adjacent_flags += 1;
            }
            adjacent_tiles.push(neighbor);
        });

        if adjacent_flags != self.neighbors(pos) {
            return UncoverResult::Safe;
        }

        let mut result = UncoverResult::Safe;
        for neighbor in adjacent_tiles {
            let tile = self.tile(neighbor);
            if !tile.uncovered && !tile.flagged && self.uncover(neighbor) == UncoverResult::HitBomb
            {
                result = UncoverResult::HitBomb;
            }
        }
        result
    }

    pub fn flag(&mut self, pos: UVec2) {
        let index = Self::coord_to_index(pos, self.size.x as usize);
        let tile = &mut self.tiles[index];
        if !tile.uncovered {
            tile.flagged = !tile.flagged;
        }
    }

    pub fn has_won(&self) -> bool {
        self.generated
            && self
                .tiles
                .iter()
                .all(|tile| tile.kind == TileKind::Bomb || tile.uncovered)
    }

    pub fn flag_bombs(&mut self) {
        for tile in &mut self.tiles {
            if tile.kind == TileKind::Bomb {
                tile.flagged = true;
            }
        }
    }

    pub fn reveal_bombs(&mut self) {
        for tile in &mut self.tiles {
            if tile.kind == TileKind::Bomb {
                tile.uncovered = true;
            }
        }
    }

    pub fn neighbors(&self, pos: UVec2) -> usize {
        let mut count = 0;
        Self::for_each_adjacent(pos, self.size, |neighbor| {
            if self.tile(neighbor).kind == TileKind::Bomb {
                count += 1;
            }
        });
        count
    }

    fn generate(&mut self, first_tile: UVec2) {
        let stride = self.size.x as usize;
        let mut candidates: Vec<usize> = (0..self.tiles.len())
            .filter(|&index| {
                let pos = Self::index_to_coord(index, stride);
                if pos.x.abs_diff(first_tile.x) <= 1 || pos.y.abs_diff(first_tile.y) <= 1 {
                    return false;
                }
                let edges = [
                    uvec2(0, 0),
                    uvec2(self.size.x - 1, 0),
                    uvec2(0, self.size.y - 1),
                    self.size - 1,
                ];
                !edges.contains(&pos)
            })
            .collect();

        let bomb_count = (self.size.x as u64
            * self.size.y as u64
            * self.difficulty.mine_density_percent()
            / 100) as usize;
        let mut rng = Rng::new();
        let bomb_count = bomb_count.min(candidates.len());
        for index in 0..bomb_count {
            let selected = rng.get_usize_range(index..candidates.len());
            candidates.swap(index, selected);
            self.tiles[candidates[index]].kind = TileKind::Bomb;
        }
    }

    fn for_each_adjacent(pos: UVec2, size: UVec2, mut visit: impl FnMut(UVec2)) {
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let x = pos.x as i32 + dx;
                let y = pos.y as i32 + dy;
                if x >= 0 && y >= 0 && x < size.x as i32 && y < size.y as i32 {
                    visit(uvec2(x as u32, y as u32));
                }
            }
        }
    }

    fn coord_to_index(coord: UVec2, stride: usize) -> usize {
        coord.y as usize * stride + coord.x as usize
    }

    fn index_to_coord(index: usize, stride: usize) -> UVec2 {
        usizevec2(index % stride, index / stride).as_uvec2()
    }

    pub fn draw(&self, camera: &api::Camera2D, world: &mut api::WorldFrame, sprites: &Sprites) {
        let (x_range, y_range) = self.visible_tile_ranges(camera);
        for y in y_range {
            for x in x_range.clone() {
                self.draw_tile(camera, world, uvec2(x, y), sprites);
            }
        }
    }

    fn visible_tile_ranges(
        &self,
        camera: &api::Camera2D,
    ) -> (std::ops::RangeInclusive<u32>, std::ops::RangeInclusive<u32>) {
        let top_left = camera.screen_to_world(ivec2(0, 0));
        let bottom_right = camera.screen_to_world(ivec2(
            camera.resolution().width as i32,
            camera.resolution().height as i32,
        ));
        let size = self.size();
        let x_range = Self::visible_tile_range(top_left.x, bottom_right.x, size.x);
        let y_range = Self::visible_tile_range(top_left.y, bottom_right.y, size.y);
        (x_range, y_range)
    }

    fn visible_tile_range(start: f32, end: f32, count: u32) -> std::ops::RangeInclusive<u32> {
        let max = count as i64 - 1;
        let first = ((start / TILE_SIZE as f32).floor() as i64 - 1).clamp(0, max) as u32;
        let last = ((end / TILE_SIZE as f32).floor() as i64 + 2).clamp(0, max) as u32;
        first..=last
    }

    fn draw_tile(
        &self,
        camera: &api::Camera2D,
        world: &mut api::WorldFrame<'_, '_>,
        pos: UVec2,
        sprites: &Sprites,
    ) {
        let tile = self.tile(pos);
        let draw_pos = (pos * TILE_SIZE).as_ivec2();
        // fixes black bars that otherwise appear between sprites when zoomed out
        let screen_min = camera.world_to_screen(draw_pos.as_vec2());
        let screen_max = camera.world_to_screen((draw_pos + TILE_SIZE as i32).as_vec2());
        let screen_size = (screen_max - screen_min).as_vec2();
        let scale = screen_size / (TILE_SIZE as f32 * camera.zoom());
        let options = SpriteOptions::default().scale(scale);

        if !tile.uncovered {
            world.sprite(&sprites.covered, draw_pos, options);
            if tile.flagged {
                world.sprite(&sprites.flag, draw_pos, options);
            }
            return;
        }

        world.sprite(&sprites.uncovered, draw_pos, options);
        match tile.kind {
            TileKind::Clear => {
                let neighbors = self.neighbors(pos);
                if neighbors != 0 {
                    let text_pos = draw_pos + TILE_SIZE as i32 / 2;
                    let color = Self::neighbor_count_color(neighbors);
                    world.text_color(neighbors.to_string(), text_pos, Anchor::Center, color);
                }
            }
            TileKind::Bomb => {
                world.sprite(&sprites.bomb, draw_pos, options);
            }
        }
    }

    fn neighbor_count_color(neighbors: usize) -> Color {
        match neighbors {
            1 => Color::Indigo,
            2 => Color::ForestGreen,
            3 => Color::TallPoppy,
            4 => Color::PurpleHeart,
            _ => Color::Bunker,
        }
    }
}
