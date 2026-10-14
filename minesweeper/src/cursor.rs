use api::{
    Button, Camera2D, Interpolation, Tween,
    glam::{UVec2, Vec2},
};

use crate::{game::TILE_SIZE, state::State};

const CAMERA_TWEEN_TICKS: usize = 10;

pub struct Cursor {
    position: UVec2,
    size: UVec2,
    camera: Camera2D,
    camera_tween: Tween<Vec2>,
    hold: Option<(Button, usize)>,
}

impl Cursor {
    const INITIAL_DELAY: usize = 40;
    const REPEAT_INTERVAL: usize = 20;
    const DIRECTIONS: [Button; 4] = [Button::Left, Button::Right, Button::Up, Button::Down];

    pub fn new(size: UVec2) -> Self {
        let position = size / 2;
        let camera_position = Self::cell_center(position);

        Self {
            position,
            size,
            camera: Camera2D::new(camera_position),
            camera_tween: Tween::new(
                camera_position,
                camera_position,
                CAMERA_TWEEN_TICKS,
                Interpolation::EaseOutCubic,
            ),
            hold: None,
        }
    }

    pub fn pos(&self) -> UVec2 {
        self.position
    }

    pub fn camera(&self) -> &Camera2D {
        &self.camera
    }

    pub fn tick(&mut self, ctx: &mut api::ScreenContext<State>) {
        let previous_position = self.position;
        self.tick_input(ctx);

        if self.position != previous_position {
            self.camera_tween.retarget(Self::cell_center(self.position));
        }
        self.camera_tween.tick();
        self.camera.set_position(self.camera_tween.current());
    }

    fn cell_center(cell: UVec2) -> Vec2 {
        cell.as_vec2() * TILE_SIZE as f32 + Vec2::splat(TILE_SIZE as f32 * 0.5)
    }

    fn tick_input(&mut self, ctx: &mut api::ScreenContext<State>) {
        let newly_pressed = Self::DIRECTIONS
            .into_iter()
            .find(|&button| ctx.input.just_pressed(button));

        if let Some(button) = newly_pressed {
            self.hold = Some((button, 0));
            self.move_cursor(button);
            return;
        }

        if let Some((button, elapsed)) = self.hold
            && ctx.input.is_pressed(button)
        {
            let elapsed = elapsed + 1;
            if elapsed >= Self::INITIAL_DELAY
                && (elapsed - Self::INITIAL_DELAY).is_multiple_of(Self::REPEAT_INTERVAL)
            {
                self.move_cursor(button);
            }
            self.hold = Some((button, elapsed));
            return;
        }

        if let Some(button) = Self::DIRECTIONS
            .into_iter()
            .find(|&button| ctx.input.is_pressed(button))
        {
            self.hold = Some((button, 0));
        } else {
            self.hold = None;
        }
    }

    fn move_cursor(&mut self, button: Button) {
        match button {
            Button::Left => self.position.x = self.position.x.saturating_sub(1),
            Button::Right => {
                self.position.x = self.position.x.saturating_add(1).min(self.size.x - 1)
            }
            Button::Up => self.position.y = self.position.y.saturating_sub(1),
            Button::Down => {
                self.position.y = self.position.y.saturating_add(1).min(self.size.y - 1)
            }
            _ => {}
        }
    }
}
