mod animation;
mod delayed_tween;
mod render;

use std::collections::VecDeque;

use animation::{
    BOMB_REVEAL_STAGGER_GROUP_SIZE, BOMB_REVEAL_STAGGER_TICKS, CHORD_STAGGER_TICKS, FlagAnimation,
    REVEAL_ANIMATION_TICKS, REVEAL_STAGGER_GROUP_SIZE, REVEAL_STAGGER_TICKS, TileAnimation,
    WIN_FLAG_ANIMATION_TICKS, WIN_FLAG_STAGGER_GROUP_SIZE, WIN_FLAG_STAGGER_TICKS,
};
use api::Rng;
use api::glam::{UVec2, usizevec2, uvec2};
use delayed_tween::DelayedTween;

use crate::{
    difficulty::Difficulty,
    tile::{Tile, TileKind},
};

#[derive(Clone)]
pub struct Board {
    size: UVec2,
    difficulty: Difficulty,
    tiles: Vec<Tile>,
    animations: Vec<TileAnimation>,
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
            animations: vec![TileAnimation::default(); size.x as usize * size.y as usize],
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
        self.uncover_after(start, 0)
    }

    fn uncover_after(&mut self, start: UVec2, delay_ticks: usize) -> UncoverResult {
        let start_index = Self::coord_to_index(start, self.size.x as usize);
        let start_tile = self.tiles[start_index];
        if start_tile.uncovered {
            return self.chord(start, delay_ticks);
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
            self.animations[start_index].reveal =
                Some(DelayedTween::new(delay_ticks, REVEAL_ANIMATION_TICKS));
            return UncoverResult::HitBomb;
        }

        self.tiles[start_index].uncovered = true;
        self.animations[start_index].reveal =
            Some(DelayedTween::new(delay_ticks, REVEAL_ANIMATION_TICKS));
        let mut pending = VecDeque::from([start]);
        let mut reveal_order = 0usize;
        while let Some(pos) = pending.pop_front() {
            if self.neighbors(pos) != 0 {
                continue;
            }

            Self::for_each_adjacent(pos, self.size, |neighbor| {
                let index = Self::coord_to_index(neighbor, self.size.x as usize);
                let tile = &mut self.tiles[index];
                if !tile.uncovered && !tile.flagged && tile.kind != TileKind::Bomb {
                    tile.uncovered = true;
                    let stagger =
                        REVEAL_STAGGER_TICKS * (1 + reveal_order / REVEAL_STAGGER_GROUP_SIZE);
                    self.animations[index].reveal = Some(DelayedTween::new(
                        delay_ticks + stagger,
                        REVEAL_ANIMATION_TICKS,
                    ));
                    reveal_order += 1;
                    pending.push_back(neighbor);
                }
            });
        }

        UncoverResult::Safe
    }

    fn chord(&mut self, pos: UVec2, delay_ticks: usize) -> UncoverResult {
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
        let mut chord_delay = 0;
        for neighbor in adjacent_tiles {
            let tile = self.tile(neighbor);
            if !tile.uncovered && !tile.flagged {
                if self.uncover_after(neighbor, delay_ticks + chord_delay) == UncoverResult::HitBomb
                {
                    result = UncoverResult::HitBomb;
                }
                chord_delay += CHORD_STAGGER_TICKS;
            }
        }
        result
    }

    pub fn flag(&mut self, pos: UVec2) {
        let index = Self::coord_to_index(pos, self.size.x as usize);
        let tile = &mut self.tiles[index];
        if !tile.uncovered {
            tile.flagged = !tile.flagged;
            self.animations[index].flag = None;
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
        let mut order = 0usize;
        for (index, tile) in self.tiles.iter_mut().enumerate() {
            if tile.kind == TileKind::Bomb && !tile.flagged {
                tile.flagged = true;
                self.animations[index].flag = Some(FlagAnimation {
                    tween: DelayedTween::new(
                        WIN_FLAG_STAGGER_TICKS * (order / WIN_FLAG_STAGGER_GROUP_SIZE),
                        WIN_FLAG_ANIMATION_TICKS,
                    ),
                });
                order += 1;
            }
        }
    }

    pub fn reveal_bombs(&mut self) {
        let mut order = 0usize;
        for (index, tile) in self.tiles.iter_mut().enumerate() {
            if tile.kind == TileKind::Bomb && !tile.uncovered {
                tile.uncovered = true;
                self.animations[index].reveal = Some(DelayedTween::new(
                    BOMB_REVEAL_STAGGER_TICKS * (order / BOMB_REVEAL_STAGGER_GROUP_SIZE),
                    REVEAL_ANIMATION_TICKS,
                ));
                order += 1;
            }
        }
    }

    pub fn tick(&mut self) {
        for animation in &mut self.animations {
            if let Some(reveal) = &mut animation.reveal {
                reveal.tick();
            }
            if let Some(flag) = &mut animation.flag {
                flag.tween.tick();
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
}
