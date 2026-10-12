use crate::core::block::BlockBehavior;

pub struct Conveyor {}

impl BlockBehavior for Conveyor {
    fn update(&mut self) {}

    fn size(&self) -> api::glam::UVec2 {
        api::glam::UVec2::splat(1)
    }

    fn name(&self) -> &'static str {
        "Conveyor"
    }

    fn can_rotate(&self) -> bool {
        true
    }
}
