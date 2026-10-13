use api::glam::{IVec2, UVec2};

use crate::core::{
    block_kind::BlockKind,
    item::{Item, ItemStack},
    world::terrain::OreKind,
};

pub use category::BlockCategory;
pub use size::BlockSize;

mod behavior;
mod category;
mod conveyor;
mod drill;
mod size;
mod smelter;
mod state;
mod storage;

use behavior::BlockBehavior;
use state::BlockState;

pub const CONVEYOR_TRAVEL_TICKS: usize = 24;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rotation {
    Up,
    Right,
    Down,
    Left,
}

impl Rotation {
    pub fn direction(self) -> IVec2 {
        match self {
            Self::Up => IVec2::NEG_Y,
            Self::Right => IVec2::X,
            Self::Down => IVec2::Y,
            Self::Left => IVec2::NEG_X,
        }
    }
}

pub struct Block {
    kind: BlockKind,
    state: BlockState,
    origin: UVec2,
    rotation: Rotation,
}

impl Block {
    pub(crate) fn new(kind: BlockKind, origin: UVec2, rotation: Rotation) -> Self {
        Self {
            kind,
            state: BlockState::new(kind),
            origin,
            rotation,
        }
    }

    pub fn kind(&self) -> BlockKind {
        self.kind
    }

    pub fn origin(&self) -> UVec2 {
        self.origin
    }

    pub fn rotation(&self) -> Rotation {
        self.rotation
    }

    pub fn footprint_cells(&self) -> u32 {
        self.kind.size().cells()
    }

    pub(crate) fn output_direction(&self) -> IVec2 {
        self.rotation.direction()
    }

    pub(crate) fn output_position(&self) -> Option<UVec2> {
        let side = self.footprint_cells();
        let direction = self.output_direction();
        let x = if direction.x < 0 {
            self.origin.x.checked_sub(1)?
        } else if direction.x > 0 {
            self.origin.x.checked_add(side)?
        } else {
            self.origin.x
        };
        let y = if direction.y < 0 {
            self.origin.y.checked_sub(1)?
        } else if direction.y > 0 {
            self.origin.y.checked_add(side)?
        } else {
            self.origin.y
        };
        Some(UVec2::new(x, y))
    }

    pub(crate) fn items(&self) -> Option<ItemStack> {
        self.state.items()
    }

    pub(crate) fn output_item(&self, tick: usize) -> Option<Item> {
        self.state.output_item(tick)
    }

    pub(crate) fn item_progress(&self, tick: usize) -> Option<f32> {
        self.state.item_progress(tick)
    }

    pub(crate) fn item_input_direction(&self) -> Option<IVec2> {
        self.state.item_input_direction()
    }

    pub(crate) fn can_accept_item(&self, item: Item) -> bool {
        self.state.can_accept(item)
    }

    pub(crate) fn accept_item(&mut self, item: Item, tick: usize, input_direction: IVec2) -> bool {
        self.state.accept(item, tick, input_direction)
    }

    pub(crate) fn take_item(&mut self) -> Option<Item> {
        self.state.take()
    }

    pub(crate) fn tick(&mut self, ore: Option<OreKind>, tick: usize, powered: bool) -> bool {
        self.state.tick(ore, tick, powered)
    }
}
