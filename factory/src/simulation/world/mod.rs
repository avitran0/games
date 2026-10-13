use std::{
    collections::{HashMap, HashSet},
    fmt,
    num::NonZeroU32,
};

use api::glam::UVec2;

use crate::simulation::{
    block::{Block, Rotation},
    block_kind::BlockKind,
    item::Item,
    world::{arena::BlockArena, chunk::Chunk, terrain::TerrainCell},
};

mod arena;
mod chunk;
mod generation;
pub mod terrain;

pub struct World {
    size: UVec2,
    chunks: Vec<Chunk>,
    blocks: BlockArena,
    seed: u64,
    stockpile: [u32; Item::COUNT],
    tick_count: usize,
    layout_revision: u64,
}

impl World {
    const MAX_SIZE: u32 = api::MAX_TILEMAP_SIDE;

    pub fn new(size: UVec2) -> Result<Self, WorldCreateError> {
        Self::generate(size, api::Rng::new().get_u64())
    }

    pub fn generate(size: UVec2, seed: u64) -> Result<Self, WorldCreateError> {
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
            seed,
            stockpile: [0; Item::COUNT],
            tick_count: 0,
            layout_revision: 0,
        };
        generation::generate(&mut world);
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

    pub fn stockpile(&self, item: Item) -> u32 {
        self.stockpile[item.index()]
    }

    /// Number of simulation steps that have completed.
    pub fn tick_count(&self) -> usize {
        self.tick_count
    }

    pub(crate) fn layout_revision(&self) -> u64 {
        self.layout_revision
    }

    pub fn terrain_at(&self, pos: UVec2) -> Option<TerrainCell> {
        let coords = self.locate(pos)?;
        Some(self.chunks[coords.chunk].terrain[coords.cell])
    }

    pub fn block_id_at(&self, pos: UVec2) -> Option<BlockId> {
        let coords = self.locate(pos)?;
        self.chunks[coords.chunk].blocks[coords.cell]
    }

    /// Connection flags in world order: right, up, left, down.
    pub(crate) fn conveyor_connections(&self, id: BlockId) -> [bool; 4] {
        let Some(block) = self
            .blocks
            .get(id)
            .filter(|block| block.kind() == BlockKind::Conveyor)
        else {
            return [false; 4];
        };
        let origin = block.origin();
        // Mindustry's local directions are right/up/left/down in world coordinates.
        let directions = [
            api::glam::IVec2::X,
            api::glam::IVec2::NEG_Y,
            api::glam::IVec2::NEG_X,
            api::glam::IVec2::Y,
        ];
        directions.map(|direction| {
            let neighbor = origin.as_ivec2() + direction;
            if neighbor.min_element() < 0 {
                return false;
            }
            let neighbor = neighbor.as_uvec2();
            let Some(neighbor_id) = self.block_id_at(neighbor) else {
                return false;
            };
            let Some(neighbor_block) = self.blocks.get(neighbor_id) else {
                return false;
            };
            let neighbor_accepts_items = matches!(
                neighbor_block.kind(),
                BlockKind::Conveyor | BlockKind::Storage | BlockKind::Smelter | BlockKind::Core
            );
            let neighbor_outputs_items = matches!(
                neighbor_block.kind(),
                BlockKind::Conveyor | BlockKind::Drill | BlockKind::Storage | BlockKind::Smelter
            );
            (block.output_position() == Some(neighbor) && neighbor_accepts_items)
                || (neighbor_outputs_items && neighbor_block.output_position() == Some(origin))
        })
    }

    pub fn blocks(&self) -> impl Iterator<Item = (BlockId, &Block)> {
        self.blocks.iter()
    }

    pub(crate) fn block(&self, id: BlockId) -> Option<&Block> {
        self.blocks.get(id)
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
        if kind == BlockKind::Core
            && self
                .blocks
                .iter()
                .any(|(_, block)| block.kind() == BlockKind::Core)
        {
            return Err(PlaceBlockError::CoreAlreadyPlaced);
        }
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
                if matches!(
                    self.chunks[coords.chunk].terrain[coords.cell].ground,
                    terrain::GroundKind::Water
                ) {
                    return Err(PlaceBlockError::Impassable { position });
                }
                cells.push(coords);
            }
        }

        let ore = (kind == BlockKind::Drill)
            .then(|| self.ore_under_footprint(origin, side))
            .flatten();
        let id = self.blocks.insert(Block::new(kind, origin, rotation, ore));
        for coords in cells {
            self.chunks[coords.chunk].blocks[coords.cell] = Some(id);
        }
        self.layout_revision = self.layout_revision.wrapping_add(1);
        Ok(id)
    }

    pub(crate) fn tick(&mut self) {
        let tick = self.tick_count;
        for (_, block) in self.blocks.iter_mut() {
            block.tick(tick);
        }

        let mut transfers = self
            .blocks
            .iter()
            .filter_map(|(from, block)| {
                let item = block.output_item(tick)?;
                let target = block.output_position()?;
                let to = self.block_id_at(target)?;
                Some((from, to, item, block.output_direction()))
            })
            .collect::<Vec<_>>();
        let transfer_targets = transfers
            .iter()
            .map(|(from, to, _, _)| (*from, *to))
            .collect::<HashMap<_, _>>();
        transfers.sort_by_key(|(from, _, _, _)| {
            let mut current = *from;
            let mut seen = HashSet::new();
            let mut depth = 0;
            while seen.insert(current) {
                let Some(&next) = transfer_targets.get(&current) else {
                    break;
                };
                depth += 1;
                current = next;
            }
            depth
        });

        for (from, to, item, input_direction) in transfers {
            if !self
                .blocks
                .get(to)
                .is_some_and(|block| block.can_accept_item(item))
            {
                continue;
            }
            let Some(item) = self
                .blocks
                .get_mut(from)
                .and_then(|block| block.take_item(tick))
            else {
                continue;
            };
            let accepted = self
                .blocks
                .get_mut(to)
                .is_some_and(|target| target.accept_item(item, tick, input_direction));
            if accepted
                && self
                    .blocks
                    .get(to)
                    .is_some_and(|block| block.kind() == BlockKind::Core)
            {
                let count = &mut self.stockpile[item.index()];
                *count = count.saturating_add(1);
            }
            if !accepted && let Some(source) = self.blocks.get_mut(from) {
                let restored = source.accept_item(item, tick, input_direction);
                debug_assert!(restored);
            }
        }
        self.tick_count = self.tick_count.wrapping_add(1);
    }

    fn ore_under_footprint(&self, origin: UVec2, side: u32) -> Option<terrain::OreKind> {
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

        self.layout_revision = self.layout_revision.wrapping_add(1);
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
    CoreAlreadyPlaced,
    OutOfBounds { position: UVec2 },
    Occupied { position: UVec2 },
    Impassable { position: UVec2 },
}

impl fmt::Display for PlaceBlockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CoreAlreadyPlaced => write!(f, "a core is already placed"),
            Self::OutOfBounds { position } => {
                write!(f, "out of bounds at ({}, {})", position.x, position.y)
            }
            Self::Occupied { position } => {
                write!(f, "occupied at ({}, {})", position.x, position.y)
            }
            Self::Impassable { position } => {
                write!(f, "impassable terrain at ({}, {})", position.x, position.y)
            }
        }
    }
}

impl std::error::Error for PlaceBlockError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockId {
    generation: NonZeroU32,
    index: u32,
}

#[cfg(test)]
mod tests {
    use api::glam::uvec2;

    use crate::simulation::{
        block::{CONVEYOR_TRAVEL_TICKS, Drill, Rotation},
        block_kind::BlockKind,
        item::Item,
    };

    use super::{BlockId, PlaceBlockError, World, WorldCreateError, terrain};

    const DRILL_EXTRACTION_TICKS: usize = Drill::EXTRACTION_PERIOD_TICKS as usize;
    const DRILL_STACK_CAPACITY: u8 = Drill::OUTPUT_CAPACITY;

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
    fn seeded_maps_are_repeatable_and_contain_varied_resources() {
        let first = World::generate(uvec2(64, 64), 0).unwrap();
        let second = World::generate(uvec2(64, 64), 0).unwrap();
        let mut ores = std::collections::HashSet::new();
        let mut terrain_kinds = std::collections::HashSet::new();
        for y in 0..64 {
            for x in 0..64 {
                let position = uvec2(x, y);
                let a = first.terrain_at(position).unwrap();
                let b = second.terrain_at(position).unwrap();
                assert_eq!(a.ground, b.ground);
                assert_eq!(a.ore, b.ore);
                terrain_kinds.insert(a.ground as u8);
                if let Some(ore) = a.ore {
                    ores.insert(ore as u8);
                }
            }
        }
        assert!(terrain_kinds.len() > 1);
        assert!(ores.len() > 1);
    }

    #[test]
    fn buildings_cannot_be_placed_on_generated_water() {
        let mut world = World::generate(uvec2(64, 64), 1234).unwrap();
        let water = (0..64)
            .flat_map(|y| (0..64).map(move |x| uvec2(x, y)))
            .find(|&position| {
                world.terrain_at(position).unwrap().ground == terrain::GroundKind::Water
            })
            .expect("the generated map should contain water");
        assert_eq!(
            world.place_block(BlockKind::Conveyor, water, Rotation::Up),
            Err(PlaceBlockError::Impassable { position: water })
        );
    }

    #[test]
    fn smelters_craft_iron_plates() {
        let mut world = World::new(uvec2(32, 32)).unwrap();
        world
            .place_block(BlockKind::Core, uvec2(16, 7), Rotation::Up)
            .unwrap();
        world
            .place_block(BlockKind::Conveyor, uvec2(16, 11), Rotation::Up)
            .unwrap();
        world
            .place_block(BlockKind::Smelter, uvec2(16, 12), Rotation::Up)
            .unwrap();
        world
            .place_block(BlockKind::Conveyor, uvec2(16, 14), Rotation::Up)
            .unwrap();
        world
            .place_block(BlockKind::Conveyor, uvec2(16, 15), Rotation::Up)
            .unwrap();
        world
            .place_block(BlockKind::Drill, uvec2(16, 16), Rotation::Up)
            .unwrap();
        for _ in 0..2_000 {
            world.tick();
        }
        assert!(world.stockpile[Item::IronPlate.index()] > 0);
    }

    #[test]
    fn worlds_allow_at_most_one_core_at_a_time() {
        // Use the protected spawn area so procedural water cannot make placement fail.
        let mut world = World::generate(uvec2(32, 32), 0).unwrap();
        let first = world
            .place_block(BlockKind::Core, uvec2(12, 12), Rotation::Up)
            .unwrap();
        assert_eq!(
            world.place_block(BlockKind::Core, uvec2(4, 12), Rotation::Up),
            Err(PlaceBlockError::CoreAlreadyPlaced)
        );
        world.remove_block_at(uvec2(13, 13)).unwrap();
        assert!(
            world
                .place_block(BlockKind::Core, uvec2(4, 12), Rotation::Up)
                .is_ok()
        );
        assert!(!world.contains_block(first));
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
    fn full_downstream_belts_are_drained_before_upstream_transfers_are_resolved() {
        let mut world = World::new(uvec2(32, 32)).unwrap();
        let upstream = world
            .place_block(BlockKind::Conveyor, uvec2(10, 12), Rotation::Up)
            .unwrap();
        let full_middle = world
            .place_block(BlockKind::Conveyor, uvec2(10, 11), Rotation::Up)
            .unwrap();
        let downstream = world
            .place_block(BlockKind::Conveyor, uvec2(10, 10), Rotation::Up)
            .unwrap();
        world.blocks.get_mut(upstream).unwrap().accept_item(
            Item::IronOre,
            0,
            api::glam::IVec2::NEG_Y,
        );
        world.blocks.get_mut(full_middle).unwrap().accept_item(
            Item::IronOre,
            0,
            api::glam::IVec2::NEG_Y,
        );

        for _ in 0..=CONVEYOR_TRAVEL_TICKS {
            world.tick();
        }

        assert_eq!(held_item(&world, upstream), None);
        assert_eq!(held_item(&world, full_middle), Some(Item::IronOre));
        assert_eq!(held_item(&world, downstream), Some(Item::IronOre));
    }

    #[test]
    fn drills_mine_iron_and_conveyors_move_items_after_the_travel_time() {
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
        assert_eq!(
            world
                .blocks()
                .find(|(id, _)| *id == first_conveyor)
                .unwrap()
                .1
                .item_progress(132),
            Some(13.0 / 24.0)
        );

        for _ in DRILL_EXTRACTION_TICKS..143 {
            world.tick();
        }
        assert_eq!(held_item(&world, first_conveyor), Some(Item::IronOre));
        assert_eq!(held_item(&world, second_conveyor), None);

        world.tick();
        assert_eq!(held_item(&world, first_conveyor), None);
        assert_eq!(held_item(&world, second_conveyor), Some(Item::IronOre));
    }

    #[test]
    fn belt_art_adapts_to_connections_and_belts_turn_by_rotation() {
        let mut world = World::new(uvec2(32, 32)).unwrap();
        let elbow = world
            .place_block(BlockKind::Conveyor, uvec2(16, 13), Rotation::Right)
            .unwrap();
        let turned_belt = world
            .place_block(BlockKind::Conveyor, uvec2(17, 13), Rotation::Right)
            .unwrap();
        world
            .place_block(BlockKind::Conveyor, uvec2(16, 14), Rotation::Up)
            .unwrap();
        world
            .place_block(BlockKind::Conveyor, uvec2(16, 15), Rotation::Up)
            .unwrap();
        world
            .place_block(BlockKind::Drill, uvec2(16, 16), Rotation::Up)
            .unwrap();

        assert_eq!(
            world.conveyor_connections(elbow),
            [true, false, false, true]
        );
        for _ in 0..200 {
            world.tick();
        }
        assert_eq!(
            world.conveyor_connections(elbow),
            [true, false, false, true]
        );
        assert_eq!(held_item(&world, turned_belt), Some(Item::IronOre));
    }

    #[test]
    fn drill_holds_output_until_a_conveyor_is_connected() {
        let mut world = World::new(uvec2(32, 32)).unwrap();
        let drill = world
            .place_block(BlockKind::Drill, uvec2(16, 16), Rotation::Up)
            .unwrap();

        let ticks = DRILL_EXTRACTION_TICKS * (DRILL_STACK_CAPACITY as usize + 2);
        for _ in 0..ticks {
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
        let mut world = World::generate(uvec2(32, 32), 0xFACA_DE01).unwrap();
        let position = (0..30)
            .flat_map(|y| (0..30).map(move |x| uvec2(x, y)))
            .find(|&origin| {
                (0..2).all(|y| {
                    (0..2).all(|x| {
                        let cell = world.terrain_at(origin + uvec2(x, y)).unwrap();
                        cell.ore.is_none() && cell.ground != terrain::GroundKind::Water
                    })
                })
            })
            .expect("map contains ore-free land");
        let drill = world
            .place_block(BlockKind::Drill, position, Rotation::Up)
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
