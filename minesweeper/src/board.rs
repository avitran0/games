use api::{
    Rng,
    glam::{UVec2, usizevec2, uvec2},
};

use crate::tile::{Tile, TileKind};

pub struct Board {
    size: UVec2,
    tiles: Vec<Tile>,
    generated: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UncoverResult {
    Safe,
    HitBomb,
}

impl Board {
    pub fn new(size: UVec2) -> Self {
        let size = size.max(UVec2::ONE);
        Self {
            size,
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

    pub fn tile(&self, pos: UVec2) -> Tile {
        self.tiles[Self::coord_to_index(pos, self.size.x as usize)]
    }

    pub fn uncover(&mut self, start: UVec2) -> UncoverResult {
        let start_index = Self::coord_to_index(start, self.size.x as usize);
        let start_tile = self.tiles[start_index];
        if start_tile.uncovered || start_tile.flagged {
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

        let mut pending = vec![start];
        while let Some(pos) = pending.pop() {
            let index = Self::coord_to_index(pos, self.size.x as usize);
            let tile = self.tiles[index];
            if tile.uncovered || tile.flagged || tile.kind == TileKind::Bomb {
                continue;
            }

            self.tiles[index].uncovered = true;
            if self.neighbors(pos) == 0 {
                pending.extend(Self::adjacent_positions(pos, self.size));
            }
        }

        UncoverResult::Safe
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
        Self::adjacent_positions(pos, self.size)
            .into_iter()
            .filter(|&neighbor| self.tile(neighbor).kind == TileKind::Bomb)
            .count()
    }

    fn generate(&mut self, first_tile: UVec2) {
        let stride = self.size.x as usize;
        let mut candidates: Vec<usize> = (0..self.tiles.len())
            .filter(|&index| {
                let pos = Self::index_to_coord(index, stride);
                pos.x.abs_diff(first_tile.x) > 1 || pos.y.abs_diff(first_tile.y) > 1
            })
            .collect();

        let bomb_count = (self.size.x as u64 * self.size.y as u64 / 10) as usize;
        let mut rng = Rng::new();
        rng.shuffle(&mut candidates);
        for index in candidates.into_iter().take(bomb_count) {
            self.tiles[index].kind = TileKind::Bomb;
        }
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
