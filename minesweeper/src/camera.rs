use api::{Camera2D, Interpolation, Tween, glam::Vec2};

pub struct CameraRig {
    camera: Camera2D,
    position_tween: Tween<Vec2>,
    zoom_tween: Tween<f32>,
}

impl Default for CameraRig {
    fn default() -> Self {
        Self::new(Vec2::ZERO)
    }
}

impl CameraRig {
    const CURSOR_MOVE_TICKS: usize = 10;
    const END_SCREEN_TWEEN_TICKS: usize = 120;
    const MIN_ZOOM: f32 = 0.0001;

    pub fn new(position: Vec2) -> Self {
        Self {
            camera: Camera2D::new(position),
            position_tween: Tween::new(
                position,
                position,
                Self::CURSOR_MOVE_TICKS,
                Interpolation::EaseOutCubic,
            ),
            zoom_tween: Tween::new(
                1.0,
                1.0,
                Self::END_SCREEN_TWEEN_TICKS,
                Interpolation::EaseInOutCubic,
            ),
        }
    }

    pub fn camera(&self) -> &Camera2D {
        &self.camera
    }

    pub fn move_to(&mut self, position: Vec2) {
        self.position_tween = Tween::new(
            self.position_tween.current(),
            position,
            Self::CURSOR_MOVE_TICKS,
            Interpolation::EaseOutCubic,
        );
    }

    pub fn frame_board(&mut self, board_size: Vec2) {
        let center = board_size * 0.5;
        let fit_zoom =
            (api::WIDTH as f32 / board_size.x).min(api::HEIGHT as f32 / board_size.y) * 0.9;
        let target_zoom = fit_zoom.clamp(Self::MIN_ZOOM, 1.0);

        self.position_tween = Tween::new(
            self.position_tween.current(),
            center,
            Self::END_SCREEN_TWEEN_TICKS,
            Interpolation::EaseInOutCubic,
        );
        self.zoom_tween = Tween::new(
            self.zoom_tween.current(),
            target_zoom,
            Self::END_SCREEN_TWEEN_TICKS,
            Interpolation::EaseInOutCubic,
        );
    }

    pub fn tick(&mut self) {
        self.position_tween.tick();
        self.zoom_tween.tick();
        self.camera.set_position(self.position_tween.current());
        self.camera.set_zoom(self.zoom_tween.current());
    }
}
