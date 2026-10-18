use api::{Camera2D, Interpolation, Resolution, Tween, glam::Vec2};

const SHAKE_TICKS: usize = 20;
const SHAKE_FREQUENCY: f32 = 2.2;
const SHAKE_VERTICAL_FREQUENCY: f32 = 1.7;
const SHAKE_HORIZONTAL_AMPLITUDE: f32 = 3.0;
const SHAKE_VERTICAL_AMPLITUDE: f32 = 2.0;

pub struct CameraRig {
    camera: Camera2D,
    position_tween: Tween<Vec2>,
    zoom_tween: Tween<f32>,
    shake: Option<Tween<f32>>,
    shake_phase: usize,
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
            shake: None,
            shake_phase: 0,
        }
    }

    pub fn camera(&self) -> &Camera2D {
        &self.camera
    }

    pub fn set_resolution(&mut self, resolution: Resolution) {
        self.camera.set_resolution(resolution);
    }

    pub fn shake(&mut self) {
        self.shake = Some(Tween::new(
            1.0,
            0.0,
            SHAKE_TICKS,
            Interpolation::EaseOutCubic,
        ));
        self.shake_phase = 0;
    }

    pub fn move_to(&mut self, position: Vec2) {
        self.position_tween = Tween::new(
            self.position_tween.current(),
            position,
            Self::CURSOR_MOVE_TICKS,
            Interpolation::EaseOutCubic,
        );
    }

    pub fn frame_board(&mut self, resolution: Resolution, board_size: Vec2) {
        let center = board_size * 0.5;
        let fit_zoom =
            (resolution.width as f32 / board_size.x).min(resolution.height as f32 / board_size.y);
        let target_zoom = fit_zoom.max(Self::MIN_ZOOM);

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

    pub fn focus_cursor(&mut self, position: Vec2) {
        self.position_tween = Tween::new(
            self.position_tween.current(),
            position,
            Self::END_SCREEN_TWEEN_TICKS,
            Interpolation::EaseInOutCubic,
        );
        self.zoom_tween = Tween::new(
            self.zoom_tween.current(),
            1.0,
            Self::END_SCREEN_TWEEN_TICKS,
            Interpolation::EaseInOutCubic,
        );
    }

    pub fn tick(&mut self) {
        self.position_tween.tick();
        self.zoom_tween.tick();

        let mut position = self.position_tween.current();
        let shake_strength = self.shake.as_mut().map(|shake| {
            shake.tick();
            shake.current()
        });
        if let Some(strength) = shake_strength {
            let phase = self.shake_phase as f32 * SHAKE_FREQUENCY;
            position += Vec2::new(
                phase.sin() * SHAKE_HORIZONTAL_AMPLITUDE,
                (phase * SHAKE_VERTICAL_FREQUENCY).cos() * SHAKE_VERTICAL_AMPLITUDE,
            ) * strength;
            self.shake_phase += 1;
            if self.shake.as_ref().is_some_and(Tween::is_finished) {
                self.shake = None;
            }
        }

        self.camera.set_position(position);
        self.camera.set_zoom(self.zoom_tween.current());
    }
}
