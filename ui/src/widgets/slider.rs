use api::Button as EngineButton;
use glam::{ivec2, uvec2};

use super::Widget;
use crate::{Rect, Response, Ui};

pub struct Slider<'a> {
    label: String,
    value: &'a mut i32,
    min: i32,
    max: i32,
}

impl<'a> Slider<'a> {
    pub fn new(label: impl Into<String>, value: &'a mut i32, min: i32, max: i32) -> Self {
        Self {
            label: label.into(),
            value,
            min,
            max,
        }
    }
}

impl Widget for Slider<'_> {
    fn show(self, ui: &mut Ui) -> Response {
        let (low, high) = (self.min.min(self.max), self.min.max(self.max));
        *self.value = (*self.value).clamp(low, high);
        let old = *self.value;
        let mut response = ui.widget_response();
        if response.focused {
            if ui.input().just_pressed(EngineButton::Left) {
                *self.value = self.value.saturating_sub(1).max(low);
                response.clicked = true;
            } else if ui.input().just_pressed(EngineButton::Right) {
                *self.value = self.value.saturating_add(1).min(high);
                response.clicked = true;
            }
        }

        let rect = ui.row_rect();
        let label = format!("{}: {}", self.label, *self.value);
        ui.draw_interactive(rect, &label, false, response);
        let bar_width = rect
            .size
            .x
            .saturating_sub(ui.style.padding.saturating_mul(2));
        let bar = Rect {
            position: rect.position + ivec2(ui.style.padding as i32, rect.size.y as i32 - 3),
            size: uvec2(bar_width, 2),
        };
        ui.draw_rect(bar, ui.style.panel_border, 1);
        let fraction = if high == low {
            1.0
        } else {
            ((*self.value as i64 - low as i64) as f64 / (high as i64 - low as i64) as f64) as f32
        };
        ui.draw_rect(
            Rect {
                size: uvec2((bar_width as f32 * fraction) as u32, 2),
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
    pub fn slider(
        &mut self,
        label: impl Into<String>,
        value: &mut i32,
        min: i32,
        max: i32,
    ) -> Response {
        self.add(Slider::new(label, value, min, max))
    }
}
