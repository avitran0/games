use std::{fmt, num::NonZeroU32};

use api::glam::UVec2;

use crate::core::{
    block::{Block, Rotation},
    block_kind::BlockKind,
    world::{arena::BlockArena, chunk::Chunk, terrain::TerrainCell},
};

mod arena;
mod chunk;
pub mod terrain;

pub struct World {
    size: UVec2,
    chunks: Vec<Chunk>,
    blocks: BlockArena,
}

impl World {
    const MAX_SIZE: u32 = api::MAX_TILEMAP_SIDE;

    pub fn new(size: UVec2) -> Result<Self, WorldCreateError> {
        if size.x == 0 || size.y == 0 {
            return Err(WorldCreateError::Empty { size });
        }
        if size.x > Self::MAX_SIZE || size.y > Self::MAX_SIZE {
            return Err(WorldCreateError::TooLarge {
                size,
                max_size: Self::MAX_SIZE,
            });
        }

        let chunk_side = Chunk::SIDE_COUNT as u32;
        if !size.x.is_multiple_of(chunk_side) || !size.y.is_multiple_of(chunk_side) {
            return Err(WorldCreateError::NotChunkAligned { size, chunk_side });
        }

        let chunk_count = (size.x / chunk_side * size.y / chunk_side) as usize;
        let mut world = Self {
            size,
            chunks: vec![Chunk::default(); chunk_count],
            blocks: BlockArena::default(),
        };
        world.generate_starting_ore();
        Ok(world)
    }

    fn locate(&self, pos: UVec2) -> Option<TileCoords> {
        if pos.x >= self.size.x || pos.y >= self.size.y {
            return None;
        }

        let chunk_side = Chunk::SIDE_COUNT as u32;
        let chunk = pos / chunk_side;
        let chunk_index = chunk.y * (self.size.x / chunk_side) + chunk.x;

        let cell = pos % chunk_side;
        let cell_index = cell.y * chunk_side + cell.x;

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

    pub fn blocks(&self) -> impl Iterator<Item = (BlockId, &Block)> {
        self.blocks.iter()
    }

    pub(crate) fn contains_block(&self, id: BlockId) -> bool {
        self.blocks.get(id).is_some()
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

    pub(crate) fn tick(&mut self) {
        let drill_ores = self
            .blocks
            .iter()
            .map(|(_, block)| self.ore_under(block))
            .collect::<Vec<_>>();
        for ((_, block), ore) in self.blocks.iter_mut().zip(drill_ores) {
            block.tick(ore);
        }

        let transfers = self
            .blocks
            .iter()
            .filter_map(|(from, block)| {
                let item = block.output_item()?;
                let target = block.output_position()?;
                let to = self.block_id_at(target)?;
                Some((from, to, item))
            })
            .collect::<Vec<_>>();

        for (from, to, item) in transfers {
            if !self
                .blocks
                .get(to)
                .is_some_and(|block| block.can_accept_item(item))
            {
                continue;
            }
            let Some(item) = self.blocks.get_mut(from).and_then(Block::take_item) else {
                continue;
            };
            let accepted = self
                .blocks
                .get_mut(to)
                .is_some_and(|target| target.accept_item(item));
            if !accepted && let Some(source) = self.blocks.get_mut(from) {
                let restored = source.accept_item(item);
                debug_assert!(restored);
            }
        }
    }

    fn ore_under(&self, block: &Block) -> Option<terrain::OreKind> {
        let origin = block.origin();
        let side = block.footprint_cells();
        for y in 0..side {
            for x in 0..side {
                if let Some(ore) = self
                    .terrain_at(UVec2::new(origin.x + x, origin.y + y))
                    .and_then(|cell| cell.ore)
                {
                    return Some(ore);
                }
            }
        }
        None
    }

    fn generate_starting_ore(&mut self) {
        const RADIUS: u32 = 4;
        let center = self.size / 2;
        let min_x = center.x.saturating_sub(RADIUS);
        let min_y = center.y.saturating_sub(RADIUS);
        let max_x = (center.x + RADIUS).min(self.size.x - 1);
        let max_y = (center.y + RADIUS).min(self.size.y - 1);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let dx = x.abs_diff(center.x);
                let dy = y.abs_diff(center.y);
                if dx * dx + dy * dy > RADIUS * RADIUS {
                    continue;
                }
                if let Some(coords) = self.locate(UVec2::new(x, y)) {
                    self.chunks[coords.chunk].terrain[coords.cell].ore =
                        Some(terrain::OreKind::Iron);
                }
            }
        }
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

#[derive(Debug, PartialEq, Eq)]
pub enum WorldCreateError {
    Empty { size: UVec2 },
    TooLarge { size: UVec2, max_size: u32 },
    NotChunkAligned { size: UVec2, chunk_side: u32 },
}

impl fmt::Display for WorldCreateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty { .. } => write!(f, "world dimensions must be non-zero"),
            Self::TooLarge { size, max_size } => write!(
                f,
                "world dimensions {}x{} exceed the maximum side length of {max_size}",
                size.x, size.y
            ),
            Self::NotChunkAligned { size, chunk_side } => write!(
                f,
                "world dimensions {}x{} must be multiples of {chunk_side}",
                size.x, size.y
            ),
        }
    }
}

impl std::error::Error for WorldCreateError {}

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

    use crate::core::{block::Rotation, block_kind::BlockKind, item::Item};

    use super::{BlockId, PlaceBlockError, World, WorldCreateError};

    const DRILL_EXTRACTION_TICKS: u16 = 120;
    const DRILL_STACK_CAPACITY: u8 = 10;

    fn held_item(world: &World, id: BlockId) -> Option<Item> {
        world.blocks().find_map(|(block_id, block)| {
            (block_id == id)
                .then(|| block.items().map(|stack| stack.item()))
                .flatten()
        })
    }

    fn held_count(world: &World, id: BlockId) -> u8 {
        world
            .blocks()
            .find_map(|(block_id, block)| {
                (block_id == id)
                    .then(|| block.items().map(|stack| stack.count()))
                    .flatten()
            })
            .unwrap_or(0)
    }

    #[test]
    fn world_creation_validates_dimensions_without_panicking() {
        assert!(matches!(
            World::new(uvec2(0, 16)),
            Err(WorldCreateError::Empty { .. })
        ));
        assert!(matches!(
            World::new(uvec2(15, 16)),
            Err(WorldCreateError::NotChunkAligned { .. })
        ));
        assert!(matches!(
            World::new(uvec2(272, 16)),
            Err(WorldCreateError::TooLarge { .. })
        ));
        assert!(World::new(uvec2(256, 16)).is_ok());
    }

    #[test]
    fn multi_cell_blocks_reserve_their_full_footprint() {
        let mut world = World::new(uvec2(16, 16)).unwrap();
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
        let mut world = World::new(uvec2(32, 32)).unwrap();
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
    fn removed_block_ids_do_not_alias_reused_slots() {
        let mut world = World::new(uvec2(16, 16)).unwrap();
        let first = world
            .place_block(BlockKind::Conveyor, uvec2(1, 1), Rotation::Up)
            .unwrap();
        world.remove_block_at(uvec2(1, 1)).unwrap();
        let second = world
            .place_block(BlockKind::Conveyor, uvec2(1, 1), Rotation::Up)
            .unwrap();

        assert_ne!(first, second);
        assert!(!world.contains_block(first));
        assert!(world.contains_block(second));
    }

    #[test]
    fn drills_mine_iron_and_conveyors_move_items_one_block_per_tick() {
        let mut world = World::new(uvec2(32, 32)).unwrap();
        let drill = world
            .place_block(BlockKind::Drill, uvec2(16, 16), Rotation::Up)
            .unwrap();
        let first_conveyor = world
            .place_block(BlockKind::Conveyor, uvec2(16, 15), Rotation::Up)
            .unwrap();
        let second_conveyor = world
            .place_block(BlockKind::Conveyor, uvec2(16, 14), Rotation::Up)
            .unwrap();

        for _ in 0..DRILL_EXTRACTION_TICKS {
            world.tick();
        }

        assert_eq!(held_item(&world, drill), None);
        assert_eq!(held_item(&world, first_conveyor), Some(Item::IronOre));
        assert_eq!(held_item(&world, second_conveyor), None);

        world.tick();
        assert_eq!(held_item(&world, first_conveyor), None);
        assert_eq!(held_item(&world, second_conveyor), Some(Item::IronOre));
    }

    #[test]
    fn drill_holds_output_until_a_conveyor_is_connected() {
        let mut world = World::new(uvec2(32, 32)).unwrap();
        let drill = world
            .place_block(BlockKind::Drill, uvec2(16, 16), Rotation::Up)
            .unwrap();

        for _ in 0..DRILL_EXTRACTION_TICKS * (DRILL_STACK_CAPACITY as u16 + 2) {
            world.tick();
        }
        assert_eq!(held_item(&world, drill), Some(Item::IronOre));
        assert_eq!(held_count(&world, drill), DRILL_STACK_CAPACITY);

        let conveyor = world
            .place_block(BlockKind::Conveyor, uvec2(16, 15), Rotation::Up)
            .unwrap();
        world.tick();
        assert_eq!(held_count(&world, drill), DRILL_STACK_CAPACITY - 1);
        assert_eq!(held_item(&world, conveyor), Some(Item::IronOre));
        assert_eq!(held_count(&world, conveyor), 1);
    }

    #[test]
    fn drills_do_not_mine_without_an_ore_deposit() {
        let mut world = World::new(uvec2(32, 32)).unwrap();
        let drill = world
            .place_block(BlockKind::Drill, uvec2(1, 1), Rotation::Up)
            .unwrap();
        for _ in 0..DRILL_EXTRACTION_TICKS * 2 {
            world.tick();
        }
        assert_eq!(held_item(&world, drill), None);
    }

    #[test]
    fn failed_placement_does_not_partially_occupy_cells() {
        let mut world = World::new(uvec2(16, 16)).unwrap();

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
        let mut world = World::new(uvec2(16, 16)).unwrap();
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
