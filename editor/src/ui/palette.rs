use crate::palette;
use eframe::egui::{self, Color32, Sense, Stroke, Ui, Vec2};

const COLUMNS: usize = 8;
const ROWS: usize = crate::palette::COLOR_COUNT.div_ceil(COLUMNS);
const SWATCH_SIZE: f32 = 16.0;
const HORIZONTAL_SPACING: f32 = 1.0;
const VERTICAL_SPACING: f32 = 3.0;

pub fn show(ui: &mut Ui, selected_color: u8) -> Option<u8> {
    let mut selection = None;
    ui.heading("Palette");
    let transparent = ui.selectable_label(selected_color == 0, "Transparent");
    if transparent.clicked() {
        selection = Some(0);
    }
    transparent.on_hover_text("Transparent (#00000000)");

    for row in 0..ROWS {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(HORIZONTAL_SPACING, VERTICAL_SPACING);
            for column in 0..COLUMNS {
                let index = row * COLUMNS + column + 1;
                if index > crate::palette::COLOR_COUNT {
                    continue;
                }
                let color = index as u8;
                let color_info = api::Color::ALL[index - 1];
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
                let is_selected = selected_color == color;
                if is_selected {
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
                let [red, green, blue] = color_info.rgb();
                let selected_label = if is_selected { "Selected · " } else { "" };
                response.on_hover_text(format!(
                    "{selected_label}{color} · {} (#{red:02X}{green:02X}{blue:02X})",
                    color_info.name()
                ));
            }
        });
        if row + 1 < ROWS {
            ui.add_space(VERTICAL_SPACING);
        }
    }
    selection
}
