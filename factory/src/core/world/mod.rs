use std::num::NonZeroU32;

use api::glam::{UVec2, uvec2};

use crate::core::world::{arena::BlockArena, chunk::Chunk, terrain::TerrainCell};

mod arena;
mod chunk;
mod terrain;

pub struct World {
    size: UVec2,
    chunks: Vec<Chunk>,
    blocks: BlockArena,
}

impl World {
    pub fn new(size: UVec2) -> Self {
        assert!(size.x < 512, "World width has to be 512 or less");
        assert!(size.y < 512, "World height has to be 512 or less");

        let size = uvec2(
            size.x.div_ceil(Chunk::SIDE_COUNT as u32),
            size.y.div_ceil(Chunk::SIDE_COUNT as u32),
        );

        Self {
            size,
            chunks: vec![Chunk::default(); (size.x * size.y) as usize],
            blocks: BlockArena::default(),
        }
    }

    fn get(&self, pos: UVec2) -> Option<TileInfo> {
        if pos.x > self.size.x || pos.y > self.size.y {
            return None;
        }

        None
    }
}

pub struct TileInfo {
    terrain: TerrainCell,
    block: Option<BlockId>,
}

#[derive(Clone, Copy, PartialEq, Hash)]
pub struct BlockId {
    generation: NonZeroU32,
    index: u32,
}
