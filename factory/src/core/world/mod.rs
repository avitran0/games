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
        assert!(
            size.x.is_multiple_of(16),
            "World width has to be a multiple of 16"
        );
        assert!(
            size.y.is_multiple_of(16),
            "World height has to be a multiple of 16"
        );

        Self {
            size,
            chunks: vec![Chunk::default(); (size.x * size.y) as usize],
            blocks: BlockArena::default(),
        }
    }

    fn locate(&self, pos: UVec2) -> Option<TileCoords> {
        if pos.x > self.size.x || pos.y > self.size.y {
            return None;
        }

        let chunk = pos / 16;
        let chunk_index = chunk.y * (self.size.x / 16) + chunk.x;

        let cell = pos % 16;
        let cell_index = cell.y * Chunk::SIDE_COUNT as u32 + cell.x;

        Some(TileCoords {
            chunk: chunk_index as usize,
            cell: cell_index as usize,
        })
    }

    fn get(&self, pos: UVec2) -> Option<TileInfo> {
        None
    }
}

struct TileCoords {
    chunk: usize,
    cell: usize,
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
