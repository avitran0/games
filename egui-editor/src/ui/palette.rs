use crate::palette;
use eframe::egui::{self, Color32, Sense, Stroke, Ui, Vec2};

const COLUMNS: usize = 8;
const SWATCH_SIZE: f32 = 16.0;
const HORIZONTAL_SPACING: f32 = 1.0;
const VERTICAL_SPACING: f32 = 3.0;

pub fn show(ui: &mut Ui, selected_color: u8) -> Option<u8> {
    let mut selection = None;
    ui.heading("Palette");

    for row in 0..8 {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(HORIZONTAL_SPACING, VERTICAL_SPACING);
            for column in 0..COLUMNS {
                let color = (row * COLUMNS + column + 1) as u8;
                let (rect, response) =
                    ui.allocate_exact_size(Vec2::splat(SWATCH_SIZE), Sense::click());
                let painter = ui.painter();
                painter.rect_filled(rect, 1.0, palette::color(color));
                painter.rect_stroke(
                    rect,
                    1.0,
                    Stroke::new(1.0, Color32::from_black_alpha(85)),
                    egui::StrokeKind::Inside,
                );
                if response.hovered() {
                    painter.rect_stroke(
                        rect.expand(1.0),
                        1.0,
                        ui.visuals().widgets.hovered.bg_stroke,
                        egui::StrokeKind::Outside,
                    );
                }
                if selected_color == color {
                    painter.rect_stroke(
                        rect.expand(1.0),
                        1.0,
                        Stroke::new(1.5, Color32::WHITE),
                        egui::StrokeKind::Outside,
                    );
                    painter.rect_stroke(
                        rect.shrink(1.0),
                        0.0,
                        Stroke::new(1.0, Color32::BLACK),
                        egui::StrokeKind::Inside,
                    );
                }
                if response.clicked() {
                    selection = Some(color);
                }
                response.on_hover_text(format!("Palette color {color}"));
            }
        });
        if row < 7 {
            ui.add_space(VERTICAL_SPACING);
        }
    }
    selection
}
