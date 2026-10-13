use eframe::egui::{self, Ui};

pub fn show(ui: &mut Ui, dirty: bool, message: &str, detail: Option<&str>) {
    ui.horizontal(|ui| {
        ui.label(if dirty { "Unsaved changes" } else { "Ready" });
        ui.separator();
        ui.label(message);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(detail) = detail {
                ui.label(detail);
            }
        });
    });
}
