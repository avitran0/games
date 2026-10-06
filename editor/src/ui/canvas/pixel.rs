use super::{CanvasView, GridBackground, active_pointer, canvas_geometry};
use crate::palette;
use api::glam::UVec2;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Ui, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PixelEdit {
    pub x: usize,
    pub y: usize,
    pub value: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelMode {
    Indexed,
    Monochrome,
}

pub fn show_pixels(
    ui: &mut Ui,
    pixels: &[u8],
    size: UVec2,
    selected_value: u8,
    mode: PixelMode,
    view: &mut CanvasView,
) -> Option<PixelEdit> {
    let width = size.x as usize;
    let height = size.y as usize;
    let (viewport, rect, cell, response) = canvas_geometry(ui, size, view);
    let painter = ui.painter_at(viewport);
    let (light, dark) = view.background.colors();
    let grid_stroke = view.background.grid_stroke();

    for y in 0..height {
        for x in 0..width {
            let value = pixels.get(y * width + x).copied().unwrap_or(0);
            let pixel_rect = Rect::from_min_size(
                Pos2::new(rect.left() + x as f32 * cell, rect.top() + y as f32 * cell),
                Vec2::splat(cell),
            );
            if !pixel_rect.intersects(viewport) {
                continue;
            }
            let fill = match (mode, value) {
                (PixelMode::Indexed, value) if value != 0 => palette::color(value),
                (PixelMode::Monochrome, value) if value != 0 => match view.background {
                    GridBackground::Light => Color32::BLACK,
                    GridBackground::Dark => Color32::WHITE,
                },
                _ if (x + y).is_multiple_of(2) => light,
                _ => dark,
            };
            painter.rect_filled(pixel_rect, 0.0, fill);
            painter.rect_stroke(
                pixel_rect,
                0.0,
                Stroke::new(0.5, grid_stroke),
                egui::StrokeKind::Inside,
            );
        }
    }
    painter.rect_stroke(
        rect,
        0.0,
        Stroke::new(1.0, ui.visuals().text_color()),
        egui::StrokeKind::Outside,
    );

    active_pointer(&response, ui).and_then(|(pointer, secondary)| {
        let x = ((pointer.x - rect.left()) / cell).floor() as usize;
        let y = ((pointer.y - rect.top()) / cell).floor() as usize;
        (rect.contains(pointer) && x < width && y < height).then_some(PixelEdit {
            x,
            y,
            value: if secondary { 0 } else { selected_value },
        })
    })
}
