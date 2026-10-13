use api::glam::UVec2;

use crate::core::{
    block::{conveyor::Conveyor, drill::Drill},
    block_kind::BlockKind,
};

pub use category::BlockCategory;
pub use size::BlockSize;

mod category;
mod conveyor;
mod drill;
mod size;

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

    pub(crate) fn tick(&mut self) {
        self.state.tick();
    }
}

enum BlockState {
    Conveyor(Conveyor),
    Drill(Drill),
}

impl BlockState {
    fn new(kind: BlockKind) -> Self {
        match kind {
            BlockKind::Conveyor => Self::Conveyor(Conveyor::new()),
            BlockKind::Drill => Self::Drill(Drill::new()),
        }
    }

    fn tick(&mut self) {
        match self {
            Self::Conveyor(conveyor) => conveyor.tick(),
            Self::Drill(drill) => drill.tick(),
        }
    }
}
