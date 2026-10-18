use glam::{IVec2, Vec2};

use crate::Resolution;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera2D {
    position: Vec2,
    zoom: f32,
    resolution: Resolution,
}

impl Camera2D {
    const MIN_ZOOM: f32 = 0.0001;

    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            zoom: 1.0,
            resolution: Resolution::default(),
        }
    }

    pub fn with_resolution(position: Vec2, resolution: Resolution) -> Self {
        Self {
            position,
            zoom: 1.0,
            resolution,
        }
    }

    pub fn resolution(&self) -> Resolution {
        self.resolution
    }

    pub fn set_resolution(&mut self, resolution: Resolution) {
        self.resolution = resolution;
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
        self.position + (position.as_vec2() - self.screen_center()) / self.zoom
    }

    fn project(&self, position: Vec2) -> Vec2 {
        self.screen_center() + (position - self.position) * self.zoom
    }

    fn screen_center(&self) -> Vec2 {
        Vec2::new(self.resolution.width as f32, self.resolution.height as f32) * 0.5
    }
}

impl Default for Camera2D {
    fn default() -> Self {
        let res = Resolution::default();
        Self::new(Vec2::new(res.width as f32, res.height as f32) * 0.5)
    }
}
