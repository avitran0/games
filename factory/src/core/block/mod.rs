use api::glam::UVec2;

use crate::core::block_kind::BlockKind;

mod conveyor;

pub enum Rotation {
    Up,
    Right,
    Down,
    Left,
}

pub struct Block {
    behavior: Box<dyn BlockBehavior>,
    rotation: Rotation,
}

pub trait BlockBehavior {
    fn update(&mut self);
    fn size(&self) -> UVec2;

    fn can_rotate(&self) -> bool {
        false
    }
}
