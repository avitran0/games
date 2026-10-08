use std::fmt::Display;

use api::Button as EngineButton;
use glam::{ivec2, uvec2};

use super::Widget;
use crate::{Rect, Response, Ui};

pub trait SliderValue: Copy + PartialOrd + Display {
    fn default_step(min: Self, max: Self) -> Self;
    fn clamp_value(self, min: Self, max: Self) -> Self;
    fn subtract_step(self, step: Self, min: Self) -> Self;
    fn add_step(self, step: Self, max: Self) -> Self;
    fn fill_width(self, min: Self, max: Self, width: u32) -> u32;
}

fn integer_fill_width(numerator: u64, denominator: u64, width: u32) -> u32 {
    if denominator == 0 {
        return width;
    }
    let mut remainder = 0_u64;
    let mut filled = 0_u32;
    for _ in 0..width {
        if numerator >= denominator - remainder {
            remainder -= denominator - numerator;
            filled += 1;
        } else {
            remainder += numerator;
        }
    }
    filled
}

macro_rules! impl_signed_slider {
    ($($type:ty),* $(,)?) => {
        $(
            impl SliderValue for $type {
                fn default_step(_: Self, _: Self) -> Self { 1 }
                fn clamp_value(self, min: Self, max: Self) -> Self { self.max(min).min(max) }
                fn subtract_step(self, step: Self, min: Self) -> Self {
                    self.saturating_sub(step).max(min)
                }
                fn add_step(self, step: Self, max: Self) -> Self {
                    self.saturating_add(step).min(max)
                }
                fn fill_width(self, min: Self, max: Self, width: u32) -> u32 {
                    let offset = (self as i64 as u64) ^ (1_u64 << 63);
                    let lower = (min as i64 as u64) ^ (1_u64 << 63);
                    let upper = (max as i64 as u64) ^ (1_u64 << 63);
                    integer_fill_width(offset - lower, upper - lower, width)
                }
            }
        )*
    };
}

macro_rules! impl_unsigned_slider {
    ($($type:ty),* $(,)?) => {
        $(
            impl SliderValue for $type {
                fn default_step(_: Self, _: Self) -> Self { 1 }
                fn clamp_value(self, min: Self, max: Self) -> Self { self.max(min).min(max) }
                fn subtract_step(self, step: Self, min: Self) -> Self {
                    self.saturating_sub(step).max(min)
                }
                fn add_step(self, step: Self, max: Self) -> Self {
                    self.saturating_add(step).min(max)
                }
                fn fill_width(self, min: Self, max: Self, width: u32) -> u32 {
                    integer_fill_width(self as u64 - min as u64, max as u64 - min as u64, width)
                }
            }
        )*
    };
}

macro_rules! impl_float_slider {
    ($($type:ty),* $(,)?) => {
        $(
            impl SliderValue for $type {
                fn default_step(min: Self, max: Self) -> Self {
                    (max - min) / 100.0
                }
                fn clamp_value(self, min: Self, max: Self) -> Self {
                    if self.is_nan() { min } else { self.max(min).min(max) }
                }
                fn subtract_step(self, step: Self, min: Self) -> Self {
                    (self - step).max(min)
                }
                fn add_step(self, step: Self, max: Self) -> Self {
                    (self + step).min(max)
                }
                fn fill_width(self, min: Self, max: Self, width: u32) -> u32 {
                    if min == max {
                        return width;
                    }
                    let (value, lower, upper) = if (max - min).is_finite() && (self - min).is_finite() {
                        (self, min, max)
                    } else {
                        (self / 2.0, min / 2.0, max / 2.0)
                    };
                    let fraction = ((value - lower) / (upper - lower)).clamp(0.0, 1.0);
                    (width as $type * fraction) as u32
                }
            }
        )*
    };
}

impl_signed_slider!(i8, i16, i32, i64, isize);
impl_unsigned_slider!(u8, u16, u32, u64, usize);
impl_float_slider!(f32, f64);

pub struct Slider<'a, T: SliderValue> {
    label: String,
    value: &'a mut T,
    min: T,
    max: T,
    step: T,
}

impl<'a, T: SliderValue> Slider<'a, T> {
    pub fn new(label: impl Into<String>, value: &'a mut T, min: T, max: T) -> Self {
        let (min, max) = if min <= max { (min, max) } else { (max, min) };
        let step = T::default_step(min, max);
        Self {
            label: label.into(),
            value,
            min,
            max,
            step,
        }
    }

    pub fn with_step(mut self, step: T) -> Self {
        self.step = step;
        self
    }
}

impl<T: SliderValue> Widget for Slider<'_, T> {
    fn show(self, ui: &mut Ui) -> Response {
        let old = *self.value;
        *self.value = self.value.clamp_value(self.min, self.max);
        let mut response = ui.widget_response();
        if response.focused {
            if ui.input().just_pressed(EngineButton::Left) {
                *self.value = self.value.subtract_step(self.step, self.min);
                response.clicked = true;
            } else if ui.input().just_pressed(EngineButton::Right) {
                *self.value = self.value.add_step(self.step, self.max);
                response.clicked = true;
            }
        }

        let rect = ui.row_rect();
        ui.draw_interactive(
            rect,
            &format!("{}: {}", self.label, *self.value),
            false,
            response,
        );
        let bar_width = rect
            .size
            .x
            .saturating_sub(ui.style.padding.saturating_mul(2));
        let bar = Rect {
            position: rect.position + ivec2(ui.style.padding as i32, rect.size.y as i32 - 3),
            size: uvec2(bar_width, 2),
        };
        ui.draw_rect(bar, ui.style.panel_border, 1);
        let filled = self.value.fill_width(self.min, self.max, bar_width);
        ui.draw_rect(
            Rect {
                size: uvec2(filled, 2),
                ..bar
            },
            ui.style.accent,
            1,
        );
        ui.advance_row();
        Response {
            changed: old != *self.value,
            ..response
        }
    }
}

impl Ui {
    pub fn slider<T: SliderValue>(
        &mut self,
        label: impl Into<String>,
        value: &mut T,
        min: T,
        max: T,
    ) -> Response {
        self.add(Slider::new(label, value, min, max))
    }
}
