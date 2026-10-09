use crate::ui::tile_preview;
use eframe::egui::{self, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use formats::TilesetDocument;

const CARD_SIZE: Vec2 = Vec2::new(60.0, 64.0);
const PREVIEW_SIZE: f32 = 46.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Select(usize),
    Add,
    Duplicate,
    Remove,
    MoveLeft,
    MoveRight,
}

pub fn show(
    ui: &mut Ui,
    tileset: &TilesetDocument,
    selected: Option<usize>,
    show_edit_actions: bool,
) -> Option<Action> {
    let mut action = None;
    ui.horizontal(|ui| {
        ui.strong("Tiles");
        ui.separator();
        ui.label(format!("{} tiles", tileset.tiles.len()));
        if show_edit_actions {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(index) = selected {
                    if ui
                        .add_enabled(
                            index + 1 < tileset.tiles.len(),
                            egui::Button::new("Move right →"),
                        )
                        .clicked()
                    {
                        action = Some(Action::MoveRight);
                    }
                    if ui
                        .add_enabled(index > 0, egui::Button::new("← Move left"))
                        .clicked()
                    {
                        action = Some(Action::MoveLeft);
                    }
                }
                if ui
                    .add_enabled(tileset.tiles.len() > 1, egui::Button::new("Remove tile"))
                    .clicked()
                {
                    action = Some(Action::Remove);
                }
                if ui
                    .add_enabled(
                        tileset.tiles.len() < u16::MAX as usize,
                        egui::Button::new("Duplicate tile"),
                    )
                    .clicked()
                {
                    action = Some(Action::Duplicate);
                }
                if ui
                    .add_enabled(
                        tileset.tiles.len() < u16::MAX as usize,
                        egui::Button::new("Add tile"),
                    )
                    .clicked()
                {
                    action = Some(Action::Add);
                }
            });
        }
    });
    egui::ScrollArea::horizontal()
        .id_salt("tileset-overview")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                for (index, tile) in tileset.tiles.iter().enumerate() {
                    let (rect, response) = ui.allocate_exact_size(CARD_SIZE, Sense::click());
                    if !rect.intersects(ui.clip_rect()) {
                        continue;
                    }
                    let painter = ui.painter_at(rect);
                    painter.rect_filled(rect, 2.0, ui.visuals().faint_bg_color);
                    let preview = Rect::from_min_size(
                        Pos2::new(rect.center().x - PREVIEW_SIZE / 2.0, rect.top() + 2.0),
                        Vec2::splat(PREVIEW_SIZE),
                    );
                    let preview_painter = painter.with_clip_rect(preview);
                    tile_preview::paint(
                        &preview_painter,
                        preview,
                        tile.pixels(),
                        tileset.tile_size,
                    );
                    painter.text(
                        Pos2::new(rect.center().x, rect.bottom() - 6.0),
                        egui::Align2::CENTER_CENTER,
                        (index + 1).to_string(),
                        egui::FontId::monospace(11.0),
                        ui.visuals().text_color(),
                    );
                    painter.rect_stroke(
                        rect,
                        2.0,
                        if selected == Some(index) {
                            Stroke::new(2.0, ui.visuals().selection.stroke.color)
                        } else {
                            Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color)
                        },
                        egui::StrokeKind::Inside,
                    );
                    if response.clicked() {
                        action = Some(Action::Select(index));
                    }
                    response.on_hover_text(format!("Tile {}", index + 1));
                }
            });
        });
    action
}
