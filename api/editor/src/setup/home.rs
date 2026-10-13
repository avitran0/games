use super::Action;
use crate::document::AssetKind;
use eframe::egui::Ui;

pub fn show(ui: &mut Ui) -> Action {
    ui.vertical_centered(|ui| {
        ui.add_space(30.0);
        ui.heading("Pixel Sprite Studio");
        ui.label("Create a new asset or open an existing pixel asset.");
        ui.add_space(24.0);
    });

    let mut action = Action::None;
    ui.columns(2, |columns| {
        columns[0].group(|ui| {
            ui.heading("Create");
            ui.add_space(8.0);
            for kind in AssetKind::ALL {
                if ui.button(format!("New {}...", kind.title())).clicked() {
                    action = Action::StartCreate(kind);
                }
            }
        });
        columns[1].group(|ui| {
            ui.heading("Open");
            ui.add_space(8.0);
            ui.label("Load a static sprite, animation, font, tileset, or tilemap.");
            ui.add_space(8.0);
            if ui.button("Open asset...").clicked() {
                action = Action::Open;
            }
        });
    });
    action
}
