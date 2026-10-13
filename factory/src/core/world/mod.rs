use std::num::NonZeroU32;

use api::glam::UVec2;

use crate::core::{
    block::{Block, Rotation},
    block_kind::BlockKind,
    world::{arena::BlockArena, chunk::Chunk, terrain::TerrainCell},
};

mod arena;
mod chunk;
pub mod command;
pub mod terrain;
pub mod view;

pub struct World {
    size: UVec2,
    chunks: Vec<Chunk>,
    blocks: BlockArena,
}

impl World {
    pub fn new(size: UVec2) -> Self {
        assert!(size.x > 0 && size.y > 0, "World must not be empty");
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
            chunks: vec![Chunk::default(); (size.x / 16 * size.y / 16) as usize],
            blocks: BlockArena::default(),
        }
    }

    fn locate(&self, pos: UVec2) -> Option<TileCoords> {
        if pos.x >= self.size.x || pos.y >= self.size.y {
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

    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub fn terrain_at(&self, pos: UVec2) -> Option<TerrainCell> {
        let coords = self.locate(pos)?;
        Some(self.chunks[coords.chunk].terrain[coords.cell])
    }

    pub fn block_id_at(&self, pos: UVec2) -> Option<BlockId> {
        let coords = self.locate(pos)?;
        self.chunks[coords.chunk].blocks[coords.cell]
    }

    pub fn place_block(
        &mut self,
        kind: BlockKind,
        origin: UVec2,
        rotation: Rotation,
    ) -> Result<BlockId, PlaceBlockError> {
        let side = kind.size().cells();
        let mut cells = Vec::with_capacity((side * side) as usize);

        for y in 0..side {
            for x in 0..side {
                let Some(x) = origin.x.checked_add(x) else {
                    return Err(PlaceBlockError::OutOfBounds { position: origin });
                };
                let Some(y) = origin.y.checked_add(y) else {
                    return Err(PlaceBlockError::OutOfBounds { position: origin });
                };
                let position = UVec2::new(x, y);
                let Some(coords) = self.locate(position) else {
                    return Err(PlaceBlockError::OutOfBounds { position });
                };
                if self.chunks[coords.chunk].blocks[coords.cell].is_some() {
                    return Err(PlaceBlockError::Occupied { position });
                }
                cells.push(coords);
            }
        }

        let id = self.blocks.insert(Block::new(kind, origin, rotation));
        for coords in cells {
            self.chunks[coords.chunk].blocks[coords.cell] = Some(id);
        }
        Ok(id)
    }

    pub fn remove_block_at(&mut self, pos: UVec2) -> Option<Block> {
        let id = self.block_id_at(pos)?;
        let block = self.blocks.get(id)?;
        let origin = block.origin();
        let side = block.footprint_cells();
        let removed = self.blocks.remove(id)?;

        for y in 0..side {
            for x in 0..side {
                let position = UVec2::new(origin.x + x, origin.y + y);
                let Some(coords) = self.locate(position) else {
                    continue;
                };
                if self.chunks[coords.chunk].blocks[coords.cell] == Some(id) {
                    self.chunks[coords.chunk].blocks[coords.cell] = None;
                }
            }
        }

        Some(removed)
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

#[derive(Debug, PartialEq, Eq)]
pub enum PlaceBlockError {
    OutOfBounds { position: UVec2 },
    Occupied { position: UVec2 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockId {
    generation: NonZeroU32,
    index: u32,
}

#[cfg(test)]
mod tests {
    use api::glam::uvec2;

    use crate::core::{block::Rotation, block_kind::BlockKind};

    use super::{PlaceBlockError, World};

    #[test]
    fn multi_cell_blocks_reserve_their_full_footprint() {
        let mut world = World::new(uvec2(16, 16));
        let id = world
            .place_block(BlockKind::Drill, uvec2(3, 3), Rotation::Up)
            .unwrap();

        assert_eq!(world.block_id_at(uvec2(3, 3)), Some(id));
        assert_eq!(world.block_id_at(uvec2(4, 4)), Some(id));
        assert_eq!(world.block_id_at(uvec2(5, 5)), None);
        assert_eq!(
            world.place_block(BlockKind::Conveyor, uvec2(4, 4), Rotation::Right),
            Err(PlaceBlockError::Occupied {
                position: uvec2(4, 4)
            })
        );
    }

    #[test]
    fn multi_cell_footprints_cross_chunk_boundaries() {
        let mut world = World::new(uvec2(32, 32));
        let id = world
            .place_block(BlockKind::Drill, uvec2(15, 15), Rotation::Up)
            .unwrap();

        for y in 15..17 {
            for x in 15..17 {
                assert_eq!(world.block_id_at(uvec2(x, y)), Some(id));
            }
        }

        world.remove_block_at(uvec2(16, 16)).unwrap();
        for y in 15..17 {
            for x in 15..17 {
                assert_eq!(world.block_id_at(uvec2(x, y)), None);
            }
        }
    }

    #[test]
    fn failed_placement_does_not_partially_occupy_cells() {
        let mut world = World::new(uvec2(16, 16));

        assert_eq!(
            world.place_block(BlockKind::Drill, uvec2(15, 15), Rotation::Up),
            Err(PlaceBlockError::OutOfBounds {
                position: uvec2(16, 15)
            })
        );
        assert_eq!(world.block_id_at(uvec2(15, 15)), None);
    }

    #[test]
    fn removing_from_any_occupied_cell_removes_the_whole_block() {
        let mut world = World::new(uvec2(16, 16));
        world
            .place_block(BlockKind::Drill, uvec2(3, 3), Rotation::Down)
            .unwrap();

        let removed = world.remove_block_at(uvec2(4, 3)).unwrap();
        assert_eq!(removed.kind(), BlockKind::Drill);
        for y in 3..5 {
            for x in 3..5 {
                assert_eq!(world.block_id_at(uvec2(x, y)), None);
            }
        }
        assert!(world.remove_block_at(uvec2(3, 3)).is_none());
        assert_eq!(world.block_id_at(uvec2(4, 3)), None);
    }
}
