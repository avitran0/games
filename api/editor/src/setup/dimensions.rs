use eframe::egui::{self, Ui};

pub fn pixel_size(ui: &mut Ui, width: &mut u32, height: &mut u32) {
    ui.horizontal(|ui| {
        ui.label("Width");
        pixel_dimension(ui, "asset-width", width);
        ui.label("x");
        ui.label("Height");
        pixel_dimension(ui, "asset-height", height);
    });
}

pub fn cell_size(ui: &mut Ui, width: &mut u32, height: &mut u32) {
    ui.horizontal(|ui| {
        ui.label("Width in cells");
        ui.add(egui::DragValue::new(width).range(1..=256));
        ui.label("x");
        ui.label("Height in cells");
        ui.add(egui::DragValue::new(height).range(1..=256));
    });
}

pub fn font_height(ui: &mut Ui, height: &mut u16) {
    ui.horizontal(|ui| {
        ui.label("Glyph height");
        egui::ComboBox::from_id_salt("font-height")
            .selected_text(format!("{height}px"))
            .show_ui(ui, |ui| {
                for value in 1..=64 {
                    ui.selectable_value(height, value, format!("{value}px"));
                }
            });
    });
}

fn pixel_dimension(ui: &mut Ui, id: &'static str, value: &mut u32) {
    egui::ComboBox::from_id_salt(id)
        .selected_text(format!("{value}px"))
        .show_ui(ui, |ui| {
            for size in (8..=96).step_by(8) {
                ui.selectable_value(value, size, format!("{size}px"));
            }
        });
}
