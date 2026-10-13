use api::glam::UVec2;

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
mod state;

use behavior::BlockBehavior;
use state::BlockState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rotation {
    Up,
    Right,
    Down,
    Left,
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

    pub(crate) fn output_position(&self) -> Option<UVec2> {
        let side = self.footprint_cells();
        match self.rotation {
            Rotation::Up => self
                .origin
                .y
                .checked_sub(1)
                .map(|y| UVec2::new(self.origin.x, y)),
            Rotation::Right => self
                .origin
                .x
                .checked_add(side)
                .map(|x| UVec2::new(x, self.origin.y)),
            Rotation::Down => self
                .origin
                .y
                .checked_add(side)
                .map(|y| UVec2::new(self.origin.x, y)),
            Rotation::Left => self
                .origin
                .x
                .checked_sub(1)
                .map(|x| UVec2::new(x, self.origin.y)),
        }
    }

    pub(crate) fn items(&self) -> Option<ItemStack> {
        self.state.items()
    }

    pub(crate) fn output_item(&self) -> Option<Item> {
        self.state.output_item()
    }

    pub(crate) fn can_accept_item(&self, item: Item) -> bool {
        self.state.can_accept(item)
    }

    pub(crate) fn accept_item(&mut self, item: Item) -> bool {
        self.state.accept(item)
    }

    pub(crate) fn take_item(&mut self) -> Option<Item> {
        self.state.take()
    }

    pub(crate) fn tick(&mut self, ore: Option<OreKind>) {
        self.state.tick(ore);
    }
}
