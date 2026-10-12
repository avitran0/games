use api::glam::UVec2;

use crate::core::block::{conveyor::Conveyor, drill::Drill};

mod conveyor;
mod drill;

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
