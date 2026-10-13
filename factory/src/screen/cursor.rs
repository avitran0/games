use api::{
    Button, Input,
    glam::{IVec2, UVec2},
};

use crate::core::{block::Rotation, block_kind::BlockKind, world::World};

pub struct Cursor {
    position: UVec2,
    selected_block: BlockKind,
    rotation: Rotation,
}

impl Cursor {
    pub fn new(world_size: UVec2) -> Self {
        Self {
            position: world_size / 2,
            selected_block: BlockKind::Conveyor,
            rotation: Rotation::Up,
        }
    }

    pub fn update(&mut self, input: &Input, world: &mut World) {
        if input.just_pressed(Button::Y) {
            self.selected_block = match self.selected_block {
                BlockKind::Conveyor => BlockKind::Drill,
                BlockKind::Drill => BlockKind::Conveyor,
            };
            self.clamp_to_world(world.size());
        }

        if input.just_pressed(Button::X) {
            self.rotation = match self.rotation {
                Rotation::Up => Rotation::Right,
                Rotation::Right => Rotation::Down,
                Rotation::Down => Rotation::Left,
                Rotation::Left => Rotation::Up,
            };
        }

        let movement = if input.just_pressed(Button::Left) {
            IVec2::NEG_X
        } else if input.just_pressed(Button::Right) {
            IVec2::X
        } else if input.just_pressed(Button::Up) {
            IVec2::NEG_Y
        } else if input.just_pressed(Button::Down) {
            IVec2::Y
        } else {
            IVec2::ZERO
        };
        let max = (world.size() - UVec2::splat(self.selected_block.size().cells())).as_ivec2();
        self.position = (self.position.as_ivec2() + movement)
            .clamp(IVec2::ZERO, max)
            .as_uvec2();

        if input.just_pressed(Button::A) {
            let _ = world.place_block(self.selected_block, self.position, self.rotation);
        } else if input.just_pressed(Button::B) {
            let _ = world.remove_block_at(self.position);
        }
    }

    pub fn position(&self) -> UVec2 {
        self.position
    }

    pub fn selected_block(&self) -> BlockKind {
        self.selected_block
    }

    pub fn rotation(&self) -> Rotation {
        self.rotation
    }

    fn clamp_to_world(&mut self, world_size: UVec2) {
        let footprint = self.selected_block.size().cells();
        self.position = self.position.min(world_size - UVec2::splat(footprint));
    }
}
