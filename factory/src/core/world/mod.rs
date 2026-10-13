use std::{fmt, num::NonZeroU32};

use api::glam::UVec2;

use crate::core::{
    block::{Block, Rotation},
    block_kind::BlockKind,
    item::Item,
    world::{arena::BlockArena, chunk::Chunk, terrain::TerrainCell},
};

mod arena;
mod chunk;
pub mod terrain;

pub struct World {
    size: UVec2,
    chunks: Vec<Chunk>,
    blocks: BlockArena,
    seed: u64,
    power: u32,
    stockpile: [u32; Item::COUNT],
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
            power: 0,
            stockpile: [0; Item::COUNT],
        };
        world.generate_terrain();
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

    pub fn power(&self) -> u32 {
        self.power
    }

    pub fn terrain_at(&self, pos: UVec2) -> Option<TerrainCell> {
        let coords = self.locate(pos)?;
        Some(self.chunks[coords.chunk].terrain[coords.cell])
    }

    pub fn block_id_at(&self, pos: UVec2) -> Option<BlockId> {
        let coords = self.locate(pos)?;
        self.chunks[coords.chunk].blocks[coords.cell]
    }

    pub(crate) fn conveyor_appearance(&self, id: BlockId) -> (usize, bool) {
        let Some(block) = self
            .blocks
            .get(id)
            .filter(|block| block.kind() == BlockKind::Conveyor)
        else {
            return (0, false);
        };
        let origin = block.origin();
        // Mindustry's local directions are right/up/left/down in world coordinates.
        let directions = [
            api::glam::IVec2::X,
            api::glam::IVec2::NEG_Y,
            api::glam::IVec2::NEG_X,
            api::glam::IVec2::Y,
        ];
        let connections = directions.map(|direction| {
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
        });

        let rotation = match block.rotation() {
            Rotation::Right => 0,
            Rotation::Up => 1,
            Rotation::Left => 2,
            Rotation::Down => 3,
        };
        let left = connections[(rotation + 3) % 4];
        let back = connections[(rotation + 2) % 4];
        let right = connections[(rotation + 1) % 4];
        let blend_case = if back && left && right {
            0
        } else if left && right {
            1
        } else if left && back {
            2
        } else if right && back {
            3
        } else if left {
            4
        } else if right {
            5
        } else {
            -1
        };

        // Match Mindustry's atlas transforms, including its mirrored curve pieces.
        match blend_case {
            0 => (3, false),
            1 => (4, false),
            2 => (2, true),
            3 => (2, false),
            4 => (1, true),
            5 => (1, false),
            _ => (0, false),
        }
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
                if matches!(
                    self.chunks[coords.chunk].terrain[coords.cell].ground,
                    terrain::GroundKind::Water
                ) {
                    return Err(PlaceBlockError::Impassable { position });
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

    pub(crate) fn tick(&mut self, tick: usize) {
        if tick.is_multiple_of(120) {
            let panels = self
                .blocks
                .iter()
                .filter(|(_, block)| block.kind() == BlockKind::SolarPanel)
                .count() as u32;
            self.power = self.power.saturating_add(panels).min(9_999);
        }

        let drill_ores = self
            .blocks
            .iter()
            .map(|(_, block)| self.ore_under(block))
            .collect::<Vec<_>>();
        let available_power = &mut self.power;
        for ((_, block), ore) in self.blocks.iter_mut().zip(drill_ores) {
            if block.tick(ore, tick, *available_power > 0) {
                *available_power -= 1;
            }
        }

        let transfers = self
            .blocks
            .iter()
            .filter_map(|(from, block)| {
                let item = block.output_item(tick)?;
                let target = block.output_position()?;
                let to = self.block_id_at(target)?;
                Some((from, to, item, block.output_direction()))
            })
            .collect::<Vec<_>>();

        for (from, to, item, input_direction) in transfers {
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

    fn generate_terrain(&mut self) {
        use terrain::{GroundKind, OreKind};
        let mut rng = api::Rng::seeded(self.seed ^ 0x9E37_79B9_7F4A_7C15);
        let center = self.size / 2;

        // Start with solid ground, then stamp irregular sandbars and shallow pools.
        for chunk in &mut self.chunks {
            for cell in &mut chunk.terrain {
                cell.ground = GroundKind::Stone;
            }
        }
        for _ in 0..(self.size.x * self.size.y / 300) {
            let origin = UVec2::new(
                rng.get_u32_range(0..self.size.x),
                rng.get_u32_range(0..self.size.y),
            );
            let radius = rng.get_u32_range(2..7);
            let ground = if rng.get_u8_range(0..5) == 0 {
                GroundKind::Water
            } else {
                GroundKind::Sand
            };
            for y in origin.y.saturating_sub(radius)..=(origin.y + radius).min(self.size.y - 1) {
                for x in origin.x.saturating_sub(radius)..=(origin.x + radius).min(self.size.x - 1)
                {
                    let dx = x.abs_diff(origin.x) as i64;
                    let dy = y.abs_diff(origin.y) as i64;
                    let edge_noise = i64::from(rng.get_u8_range(0..4)) - 1;
                    let distance = dx * dx + dy * dy;
                    let position = UVec2::new(x, y);
                    let spawn_distance = position
                        .x
                        .abs_diff(center.x)
                        .max(position.y.abs_diff(center.y));
                    if distance <= (radius * radius) as i64 + edge_noise
                        && spawn_distance > 12
                        && let Some(coords) = self.locate(position)
                    {
                        self.chunks[coords.chunk].terrain[coords.cell].ground = ground;
                    }
                }
            }
        }

        for y in center.y.saturating_sub(4)..=(center.y + 4).min(self.size.y - 1) {
            for x in center.x.saturating_sub(4)..=(center.x + 4).min(self.size.x - 1) {
                let dx = x.abs_diff(center.x) as i32;
                let dy = y.abs_diff(center.y) as i32;
                if dx * dx + dy * dy <= 16 {
                    self.set_ore(UVec2::new(x, y), OreKind::Iron);
                }
            }
        }

        for _ in 0..(self.size.x * self.size.y / 180) {
            let origin = UVec2::new(
                rng.get_u32_range(0..self.size.x),
                rng.get_u32_range(0..self.size.y),
            );
            let radius = rng.get_u32_range(2..5);
            let ore = match rng.get_u8_range(0..10) {
                0..=5 => OreKind::Iron,
                6..=8 => OreKind::Coal,
                _ => OreKind::Titanium,
            };
            for y in origin.y.saturating_sub(radius)..=(origin.y + radius).min(self.size.y - 1) {
                for x in origin.x.saturating_sub(radius)..=(origin.x + radius).min(self.size.x - 1)
                {
                    let dx = x.abs_diff(origin.x) as i64;
                    let dy = y.abs_diff(origin.y) as i64;
                    if dx * dx + dy * dy <= (radius * radius) as i64
                        && (UVec2::new(x, y).as_ivec2() - center.as_ivec2()).length_squared() > 100
                    {
                        self.set_ore(UVec2::new(x, y), ore);
                    }
                }
            }
        }
    }

    fn set_ore(&mut self, position: UVec2, ore: terrain::OreKind) {
        let Some(coords) = self.locate(position) else {
            return;
        };
        let cell = &mut self.chunks[coords.chunk].terrain[coords.cell];
        if !matches!(cell.ground, terrain::GroundKind::Water) {
            cell.ore = Some(ore);
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
    Impassable { position: UVec2 },
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

    use super::{BlockId, PlaceBlockError, World, WorldCreateError, terrain};

    const DRILL_EXTRACTION_TICKS: usize = 120;
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
    fn solar_power_runs_smelters_and_makes_iron_plates() {
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
        world
            .place_block(BlockKind::SolarPanel, uvec2(20, 16), Rotation::Up)
            .unwrap();

        for tick in 0..2_000 {
            world.tick(tick);
        }
        assert!(world.stockpile[Item::IronPlate.index()] > 0);
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

        for tick in 0..DRILL_EXTRACTION_TICKS {
            world.tick(tick);
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
            Some(0.5)
        );

        for tick in DRILL_EXTRACTION_TICKS..144 {
            world.tick(tick);
        }
        assert_eq!(held_item(&world, first_conveyor), Some(Item::IronOre));
        assert_eq!(held_item(&world, second_conveyor), None);

        world.tick(144);
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

        assert_eq!(world.conveyor_appearance(elbow), (1, true));
        for tick in 0..200 {
            world.tick(tick);
        }
        assert_eq!(world.conveyor_appearance(elbow), (1, true));
        assert_eq!(held_item(&world, turned_belt), Some(Item::IronOre));
    }

    #[test]
    fn drill_holds_output_until_a_conveyor_is_connected() {
        let mut world = World::new(uvec2(32, 32)).unwrap();
        let drill = world
            .place_block(BlockKind::Drill, uvec2(16, 16), Rotation::Up)
            .unwrap();

        let ticks = DRILL_EXTRACTION_TICKS * (DRILL_STACK_CAPACITY as usize + 2);
        for tick in 0..ticks {
            world.tick(tick);
        }
        assert_eq!(held_item(&world, drill), Some(Item::IronOre));
        assert_eq!(held_count(&world, drill), DRILL_STACK_CAPACITY);

        let conveyor = world
            .place_block(BlockKind::Conveyor, uvec2(16, 15), Rotation::Up)
            .unwrap();
        world.tick(ticks);
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
        for tick in 0..DRILL_EXTRACTION_TICKS * 2 {
            world.tick(tick);
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
