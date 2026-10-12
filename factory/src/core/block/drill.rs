use crate::core::block::BlockBehavior;

pub struct Drill {

}

impl BlockBehavior for Drill {
    fn update(&mut self) {

    }

    fn size(&self) -> api::glam::UVec2 {
        api::glam::UVec2::splat(2)
    }

    fn name(&self) -> &'static str {
        "Drill"
    }
}
