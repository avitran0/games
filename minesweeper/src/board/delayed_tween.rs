use api::{Interpolation, Tween};

#[derive(Clone)]
pub(in crate::board) struct DelayedTween {
    delay_ticks: usize,
    tween: Tween<f32>,
}

impl DelayedTween {
    pub(in crate::board) fn new(delay_ticks: usize, duration: usize) -> Self {
        Self {
            delay_ticks,
            tween: Tween::new(0.0, 1.0, duration, Interpolation::EaseOutCubic),
        }
    }

    pub(in crate::board) fn tick(&mut self) {
        if self.delay_ticks > 0 {
            self.delay_ticks -= 1;
        }
        if self.delay_ticks == 0 {
            self.tween.tick();
        }
    }

    pub(in crate::board) fn current(&self) -> f32 {
        if self.delay_ticks > 0 {
            0.0
        } else {
            self.tween.current()
        }
    }

    pub(in crate::board) fn is_finished(&self) -> bool {
        self.delay_ticks == 0 && self.tween.is_finished()
    }
}
