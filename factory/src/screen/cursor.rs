use api::{
    Button, Input,
    glam::{IVec2, UVec2},
};

use crate::simulation::{
    block::Rotation,
    block_kind::BlockKind,
    world::{PlaceBlockError, World},
};

const INITIAL_MOVE_REPEAT_DELAY: usize = 20;
const MOVE_REPEAT_INTERVAL: usize = 10;

pub struct Cursor {
    position: UVec2,
    rotation: Rotation,
    held_direction: Option<Button>,
    held_ticks: usize,
}

impl Cursor {
    pub fn new(world_size: UVec2) -> Self {
        Self {
            position: world_size / 2,
            rotation: Rotation::Up,
            held_direction: None,
            held_ticks: 0,
        }
    }

    pub fn update(
        &mut self,
        input: &Input,
        world: &mut World,
        picked_block: Option<BlockKind>,
    ) -> Option<Result<(), PlaceBlockError>> {
        if input.just_pressed(Button::X) {
            self.rotation = self.rotation.clockwise();
        }

        let movement = self.movement(input);
        let footprint = picked_block.map_or(1, |block| block.size().cells());
        let max = (world.size() - UVec2::splat(footprint)).as_ivec2();
        self.position = (self.position.as_ivec2() + movement)
            .clamp(IVec2::ZERO, max)
            .as_uvec2();

        if input.just_pressed(Button::A) {
            if let Some(block) = picked_block {
                return Some(
                    world
                        .place_block(block, self.position, self.rotation)
                        .map(|_| ()),
                );
            }
        } else if input.just_pressed(Button::B) {
            let _ = world.remove_block_at(self.position);
        }
        None
    }

    fn movement(&mut self, input: &Input) -> IVec2 {
        let direction = [Button::Left, Button::Right, Button::Up, Button::Down]
            .into_iter()
            .find(|&button| input.is_pressed(button));
        self.movement_for(direction)
    }

    fn movement_for(&mut self, direction: Option<Button>) -> IVec2 {
        let Some(direction) = direction else {
            self.held_direction = None;
            self.held_ticks = 0;
            return IVec2::ZERO;
        };

        if self.held_direction != Some(direction) {
            self.held_direction = Some(direction);
            self.held_ticks = 0;
            return movement_delta(direction);
        }

        self.held_ticks = self.held_ticks.saturating_add(1);
        let repeat_ticks = self.held_ticks.saturating_sub(INITIAL_MOVE_REPEAT_DELAY);
        if self.held_ticks >= INITIAL_MOVE_REPEAT_DELAY
            && repeat_ticks.is_multiple_of(MOVE_REPEAT_INTERVAL)
        {
            movement_delta(direction)
        } else {
            IVec2::ZERO
        }
    }

    pub fn position(&self) -> UVec2 {
        self.position
    }

    pub fn rotation(&self) -> Rotation {
        self.rotation
    }
}

fn movement_delta(direction: Button) -> IVec2 {
    match direction {
        Button::Left => IVec2::NEG_X,
        Button::Right => IVec2::X,
        Button::Up => IVec2::NEG_Y,
        Button::Down => IVec2::Y,
        _ => IVec2::ZERO,
    }
}

#[cfg(test)]
mod tests {
    use api::glam::{IVec2, uvec2};

    use super::{Cursor, INITIAL_MOVE_REPEAT_DELAY, MOVE_REPEAT_INTERVAL};
    use api::Button;

    #[test]
    fn held_direction_repeats_after_delay_and_on_interval() {
        let mut cursor = Cursor::new(uvec2(16, 16));
        assert_eq!(cursor.movement_for(Some(Button::Right)), IVec2::X);

        for _ in 1..INITIAL_MOVE_REPEAT_DELAY {
            assert_eq!(cursor.movement_for(Some(Button::Right)), IVec2::ZERO);
        }
        assert_eq!(cursor.movement_for(Some(Button::Right)), IVec2::X);

        for _ in 1..MOVE_REPEAT_INTERVAL {
            assert_eq!(cursor.movement_for(Some(Button::Right)), IVec2::ZERO);
        }
        assert_eq!(cursor.movement_for(Some(Button::Right)), IVec2::X);
    }

    #[test]
    fn releasing_or_changing_direction_repeats_immediately() {
        let mut cursor = Cursor::new(uvec2(16, 16));
        assert_eq!(cursor.movement_for(Some(Button::Left)), IVec2::NEG_X);
        for _ in 0..INITIAL_MOVE_REPEAT_DELAY {
            cursor.movement_for(Some(Button::Left));
        }
        assert_eq!(cursor.movement_for(Some(Button::Down)), IVec2::Y);
        assert_eq!(cursor.movement_for(None), IVec2::ZERO);
        assert_eq!(cursor.movement_for(Some(Button::Up)), IVec2::NEG_Y);
    }
}
