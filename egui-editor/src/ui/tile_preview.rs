use crate::palette;
use api::glam::UVec2;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};

pub fn paint(painter: &egui::Painter, rect: Rect, pixels: &[u8], size: UVec2) {
    paint_checker(painter, rect);
    let width = size.x as usize;
    let height = size.y as usize;
    if width == 0 || height == 0 {
        return;
    }
    let scale = (rect.width() / width as f32).min(rect.height() / height as f32);
    let image_size = Vec2::new(width as f32 * scale, height as f32 * scale);
    let image = Rect::from_center_size(rect.center(), image_size);
    for y in 0..height {
        for x in 0..width {
            let value = pixels.get(y * width + x).copied().unwrap_or(0);
            if value == 0 {
                continue;
            }
            painter.rect_filled(
                Rect::from_min_size(
                    Pos2::new(
                        image.left() + x as f32 * scale,
                        image.top() + y as f32 * scale,
                    ),
                    Vec2::splat(scale),
                ),
                0.0,
                palette::color(value),
            );
        }
    }
}

pub fn paint_empty(painter: &egui::Painter, rect: Rect) {
    paint_checker(painter, rect);
    painter.line_segment(
        [rect.left_bottom(), rect.right_top()],
        Stroke::new(1.5, Color32::WHITE),
    );
}

fn paint_checker(painter: &egui::Painter, rect: Rect) {
    const CELLS: usize = 4;
    let cell_width = rect.width() / CELLS as f32;
    let cell_height = rect.height() / CELLS as f32;
    for y in 0..CELLS {
        for x in 0..CELLS {
            painter.rect_filled(
                Rect::from_min_size(
                    Pos2::new(
                        rect.left() + x as f32 * cell_width,
                        rect.top() + y as f32 * cell_height,
                    ),
                    Vec2::new(cell_width, cell_height),
                ),
                0.0,
                if (x + y) % 2 == 0 {
                    Color32::from_gray(225)
                } else {
                    Color32::from_gray(185)
                },
            );
        }
    }
}
