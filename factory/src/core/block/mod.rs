use api::glam::UVec2;

use crate::core::{
    block::{conveyor::Conveyor, drill::Drill},
    block_kind::BlockKind,
};

pub use size::BlockSize;

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
        self.state.size().cells()
    }
}

pub enum BlockState {
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

    pub fn size(&self) -> BlockSize {
        match self {
            Self::Conveyor(_) => BlockSize::Single,
            Self::Drill(_) => BlockSize::Double,
        }
    }
}
