use crate::simulation::world::{BlockId, terrain::TerrainCell};

#[derive(Clone)]
pub struct Chunk {
    pub terrain: [TerrainCell; Self::CELL_COUNT],
    pub blocks: [Option<BlockId>; Self::CELL_COUNT],
}

impl Chunk {
    pub const SIDE_COUNT: usize = 16;
    pub const CELL_COUNT: usize = Self::SIDE_COUNT * Self::SIDE_COUNT;
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            terrain: [TerrainCell::default(); Self::CELL_COUNT],
            blocks: [None; Self::CELL_COUNT],
        }
    }
}
