use api::glam::UVec2;

use crate::core::block::{conveyor::Conveyor, drill::Drill, size::BlockSize};

mod conveyor;
mod drill;
mod size;

pub enum Rotation {
    Up,
    Right,
    Down,
    Left,
}

pub struct Block {
    state: BlockState,
    origin: UVec2,
    rotation: Rotation,
}

pub enum BlockState {
    Conveyor(Conveyor),
    Drill(Drill),
}

impl BlockState {
    pub fn size(&self) -> BlockSize {
        match self {
            Self::Conveyor(_) => BlockSize::Single,
            Self::Drill(_) => BlockSize::Double,
        }
    }
}
