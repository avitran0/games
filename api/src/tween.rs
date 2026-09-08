use std::{
    f32::consts::PI,
    ops::{Add, Mul},
};

pub trait Tweenable: Copy + Add<Output = Self> + Mul<f32, Output = Self> {}
impl<T: Copy + Add<Output = T> + Mul<f32, Output = T>> Tweenable for T {}

pub struct Tween<T: Tweenable> {
    start: T,
    end: T,
    current_tick: usize,
    ticks: usize,
    interpolation: Interpolation,
}

impl<T: Tweenable> Tween<T> {
    pub fn new(start: T, end: T, ticks: usize, interpolation: Interpolation) -> Self {
        Self {
            start,
            end,
            current_tick: 0,
            ticks,
            interpolation,
        }
    }

    pub fn tick(&mut self) {
        self.current_tick = (self.current_tick + 1).min(self.ticks);
    }

    pub fn current(&self) -> T {
        self.interpolation.interpolate(
            self.start,
            self.end,
            self.current_tick as f32 / self.ticks as f32,
        )
    }

    pub fn reset(&mut self) {
        self.current_tick = 0;
    }

    pub fn start(&self) -> T {
        self.start
    }

    pub fn end(&self) -> T {
        self.end
    }

    pub fn set_start(&mut self, start: T) {
        self.start = start;
    }

    pub fn set_end(&mut self, end: T) {
        self.end = end;
    }

    pub fn retarget(&mut self, end: T) {
        self.start = self.current();
        self.end = end;
        self.current_tick = 0;
    }

    pub fn is_finished(&self) -> bool {
        self.current_tick >= self.ticks
    }
}

pub enum Interpolation {
    // TODO: Add the easing functions from https://easings.net.
    Linear,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
}

impl Interpolation {
    fn ease(&self, t: f32) -> f32 {
        match self {
            Self::Linear => t,
            Self::EaseInSine => 1.0 - (t * PI / 2.0).cos(),
            Self::EaseOutSine => (t * PI / 2.0).sin(),
            Self::EaseInOutSine => -((t * PI).cos() - 1.0) / 2.0,
            Self::EaseInQuad => t.powi(2),
            Self::EaseOutQuad => 1.0 - (1.0 - t) * (1.0 - t),
            Self::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t.powi(2)
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Self::EaseInCubic => t.powi(3),
            Self::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            Self::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t.powi(3)
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
        }
    }

    fn interpolate<T: Tweenable>(&self, start: T, end: T, fraction: f32) -> T {
        let t = fraction.clamp(0.0, 1.0);
        let eased = self.ease(t);
        start * (1.0 - eased) + (end * eased)
    }
}

#[cfg(test)]
mod test {
    use super::Interpolation;

    macro_rules! test_interp {
        ($name:ident, $interp:expr, $values:expr) => {
            #[test]
            fn $name() {
                const EXPECTED: &[f32] = &$values;
                let len = EXPECTED.len();
                let keys: Vec<f32> = (0..len)
                    .map(|idx| {
                        if idx == 0 {
                            0.0
                        } else {
                            idx as f32 / (len - 1) as f32
                        }
                    })
                    .collect();

                for (index, key) in keys.iter().enumerate() {
                    let eased = $interp.ease(*key);
                    let expected = EXPECTED[index];
                    assert!((eased - expected).abs() < 0.01);
                }
            }
        };
    }

    test_interp!(
        linear,
        Interpolation::Linear,
        [0.0, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875, 1.0]
    );
    test_interp!(
        in_sine,
        Interpolation::EaseInSine,
        [0.0, 0.02, 0.08, 0.17, 0.29, 0.44, 0.62, 0.80, 1.0]
    );
    test_interp!(
        out_sine,
        Interpolation::EaseOutSine,
        [0.0, 0.2, 0.38, 0.56, 0.71, 0.83, 0.92, 0.98, 1.0]
    );
    test_interp!(
        in_out_sine,
        Interpolation::EaseInOutSine,
        [0.0, 0.04, 0.15, 0.31, 0.5, 0.69, 0.85, 0.96, 1.0]
    );
    test_interp!(
        in_quad,
        Interpolation::EaseInQuad,
        [0.0, 0.02, 0.06, 0.14, 0.25, 0.39, 0.56, 0.77, 1.0]
    );
    test_interp!(
        out_quad,
        Interpolation::EaseOutQuad,
        [0.0, 0.23, 0.44, 0.61, 0.75, 0.86, 0.94, 0.98, 1.0]
    );
    test_interp!(
        in_out_quad,
        Interpolation::EaseInOutQuad,
        [0.0, 0.03, 0.13, 0.28, 0.50, 0.72, 0.88, 0.97, 1.0]
    );
    test_interp!(
        in_cubic,
        Interpolation::EaseInCubic,
        [0.0, 0.0, 0.02, 0.05, 0.13, 0.24, 0.42, 0.67, 1.0]
    );
    test_interp!(
        out_cubic,
        Interpolation::EaseOutCubic,
        [0.0, 0.33, 0.58, 0.76, 0.88, 0.95, 0.98, 1.0, 1.0]
    );
    test_interp!(
        in_out_cubic,
        Interpolation::EaseInOutCubic,
        [0.0, 0.01, 0.06, 0.21, 0.5, 0.79, 0.94, 0.99, 1.0]
    );
}
