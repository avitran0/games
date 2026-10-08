use crate::ui::tile_preview;
use eframe::egui::{self, Rect, Stroke, Ui, Vec2};
use formats::TilesetDocument;

const COLUMNS: usize = 3;
const BUTTON_SIZE: f32 = 44.0;
const PREVIEW_SIZE: f32 = 34.0;

pub fn show(ui: &mut Ui, tileset: &TilesetDocument, selected_id: u32) -> Option<u32> {
    let mut selected = None;
    ui.heading("Tiles");
    egui::Grid::new("tile-palette")
        .num_columns(COLUMNS)
        .spacing(Vec2::splat(3.0))
        .show(ui, |ui| {
            for id in 0..=tileset.tiles.len() as u32 {
                let selected_button = ui
                    .add_sized(
                        [BUTTON_SIZE, BUTTON_SIZE],
                        egui::Button::new("").selected(id == selected_id),
                    )
                    .on_hover_text(if id == 0 {
                        "Empty cell".to_owned()
                    } else {
                        format!("Tile {id}")
                    });
                let preview = Rect::from_center_size(
                    selected_button.rect.center() - Vec2::new(0.0, 3.0),
                    Vec2::splat(PREVIEW_SIZE),
                );
                let painter = ui.painter().with_clip_rect(preview);
                if id == 0 {
                    tile_preview::paint_empty(&painter, preview);
                } else if let Some(tile) = tileset.tiles.get(id as usize - 1) {
                    tile_preview::paint(&painter, preview, tile.pixels(), tileset.tile_size);
                }
                painter.text(
                    selected_button.rect.left_bottom() + Vec2::new(3.0, -2.0),
                    egui::Align2::LEFT_BOTTOM,
                    id.to_string(),
                    egui::FontId::monospace(9.0),
                    ui.visuals().text_color(),
                );
                if id == selected_id {
                    painter.rect_stroke(
                        selected_button.rect,
                        1.0,
                        Stroke::new(1.5, ui.visuals().selection.stroke.color),
                        egui::StrokeKind::Outside,
                    );
                }
                if selected_button.clicked() {
                    selected = Some(id);
                }
                if id as usize % COLUMNS == COLUMNS - 1 {
                    ui.end_row();
                }
            }
        });
    selected
}
