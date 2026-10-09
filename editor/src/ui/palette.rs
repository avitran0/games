use crate::palette;
use eframe::egui::{self, Color32, Sense, Stroke, Ui, Vec2};

const COLUMNS: usize = 16;
const ENTRY_COUNT: usize = crate::palette::COLOR_COUNT + 1;
const ROWS: usize = ENTRY_COUNT.div_ceil(COLUMNS);
const SWATCH_SIZE: f32 = 16.0;
const HORIZONTAL_SPACING: f32 = 1.0;
const VERTICAL_SPACING: f32 = 3.0;

pub fn show(ui: &mut Ui, selected_color: u8) -> Option<u8> {
    let mut selection = None;
    ui.heading("Palette");

    for row in 0..ROWS {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(HORIZONTAL_SPACING, VERTICAL_SPACING);
            for column in 0..COLUMNS {
                let index = row * COLUMNS + column;
                if index >= ENTRY_COUNT {
                    continue;
                }
                let color = index as u8;
                let (rect, response) =
                    ui.allocate_exact_size(Vec2::splat(SWATCH_SIZE), Sense::click());
                let painter = ui.painter();
                if color == 0 {
                    paint_transparency_checkerboard(painter, rect);
                } else {
                    painter.rect_filled(rect, 1.0, palette::color(color));
                }
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

                if color == 0 {
                    let selected_label = if is_selected { "Selected · " } else { "" };
                    response.on_hover_text(format!("{selected_label}Transparent (#00000000)"));
                } else {
                    let color_info = api::Color::ALL[index - 1];
                    let [red, green, blue] = color_info.rgb();
                    let selected_label = if is_selected { "Selected · " } else { "" };
                    response.on_hover_text(format!(
                        "{selected_label}{color} · {} (#{red:02X}{green:02X}{blue:02X})",
                        color_info.name()
                    ));
                }
            }
        });
        if row + 1 < ROWS {
            ui.add_space(VERTICAL_SPACING);
        }
    }
    selection
}

fn paint_transparency_checkerboard(painter: &egui::Painter, rect: egui::Rect) {
    const CHECK_SIZE: f32 = 4.0;
    const CHECKS_PER_SIDE: usize = 4;

    painter.rect_filled(rect, 1.0, Color32::WHITE);
    for y in 0..CHECKS_PER_SIDE {
        for x in 0..CHECKS_PER_SIDE {
            if (x + y) % 2 == 0 {
                let position = rect.min + Vec2::new(x as f32 * CHECK_SIZE, y as f32 * CHECK_SIZE);
                painter.rect_filled(
                    egui::Rect::from_min_size(position, Vec2::splat(CHECK_SIZE)),
                    0.0,
                    Color32::LIGHT_GRAY,
                );
            }
        }
    }
}
