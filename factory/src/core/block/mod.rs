use api::glam::UVec2;

mod conveyor;
mod drill;

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
    fn name(&self) -> &'static str;

    fn can_rotate(&self) -> bool {
        false
    }
}
