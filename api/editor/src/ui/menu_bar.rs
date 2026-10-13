use crate::document::AssetKind;
use eframe::egui::{self, Ui};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    New(AssetKind),
    Open,
    Save,
    SaveAs,
    Back,
    Quit,
}

pub fn show(ui: &mut Ui, has_document: bool) -> Option<Action> {
    let mut action = None;

    egui::MenuBar::new().ui(ui, |ui| {
        ui.menu_button("File", |ui| {
            ui.menu_button("New", |ui| {
                for kind in AssetKind::ALL {
                    if ui.button(kind.title()).clicked() {
                        action = Some(Action::New(kind));
                        ui.close();
                    }
                }
            });

            if ui.button("Open").clicked() {
                action = Some(Action::Open);
                ui.close();
            }
            ui.separator();

            if ui
                .add_enabled(has_document, egui::Button::new("Save"))
                .clicked()
            {
                action = Some(Action::Save);
                ui.close();
            }

            if ui
                .add_enabled(has_document, egui::Button::new("Save As"))
                .clicked()
            {
                action = Some(Action::SaveAs);
                ui.close();
            }
            ui.separator();

            if ui.button("Back to setup").clicked() {
                action = Some(Action::Back);
                ui.close();
            }

            if ui.button("Quit").clicked() {
                action = Some(Action::Quit);
                ui.close();
            }
        });
    });

    action
}
