use glam::ivec2;

use super::Widget;
use crate::{Rect, Response, Ui};

pub struct ProgressBar {
    fraction: f32,
    label: String,
}

impl ProgressBar {
    pub fn new(fraction: f32) -> Self {
        Self {
            fraction,
            label: String::new(),
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
}

impl Widget for ProgressBar {
    fn show(self, ui: &mut Ui) -> Response {
        let rect = ui.row_rect();
        ui.draw_bordered_rect(
            rect,
            ui.style.widget,
            ui.style.panel_border,
            ui.style.border_width,
            ui.style.corner_radius,
        );
        let inset = ui.style.padding.min(rect.size.x / 2);
        let inner_width = rect.size.x.saturating_sub(inset.saturating_mul(2));
        let fill_width = (inner_width as f32 * self.fraction.clamp(0.0, 1.0)) as u32;
        ui.draw_rect(
            Rect::new(
                rect.position.x + inset as i32,
                rect.position.y + 2,
                fill_width,
                rect.size.y.saturating_sub(4),
            ),
            ui.style.accent,
            ui.style.corner_radius,
        );
        if !self.label.is_empty() {
            ui.draw_text(
                self.label,
                rect.position + ivec2(inset as i32 + 2, 2),
                ui.style.text,
            );
        }
        ui.advance_row();
        Response::default()
    }
}

impl Ui {
    pub fn progress_bar(&mut self, fraction: f32, label: impl Into<String>) -> Response {
        self.add(ProgressBar::new(fraction).label(label))
    }
}
