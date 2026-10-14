use glam::{IVec2, Vec2};

use crate::{HEIGHT, WIDTH};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera2D {
    position: Vec2,
    zoom: f32,
}

impl Camera2D {
    const MIN_ZOOM: f32 = 0.0001;

    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            zoom: 1.0,
        }
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = if zoom.is_finite() {
            zoom.max(Self::MIN_ZOOM)
        } else {
            1.0
        };
    }

    pub fn world_to_screen(&self, position: Vec2) -> IVec2 {
        self.project(position).round().as_ivec2()
    }

    pub fn screen_to_world(&self, position: IVec2) -> Vec2 {
        self.position + (position.as_vec2() - Self::screen_center()) / self.zoom
    }

    fn project(&self, position: Vec2) -> Vec2 {
        Self::screen_center() + (position - self.position) * self.zoom
    }

    fn screen_center() -> Vec2 {
        Vec2::new(WIDTH as f32, HEIGHT as f32) * 0.5
    }
}

impl Default for Camera2D {
    fn default() -> Self {
        Self::new(Self::screen_center())
    }
}
