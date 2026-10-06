use super::{AssetSpec, dimensions};
use api::glam::uvec2;
use eframe::egui::Ui;

pub struct Screen {
    width: u32,
    height: u32,
}

impl Default for Screen {
    fn default() -> Self {
        Self {
            width: 16,
            height: 16,
        }
    }
}

impl Screen {
    pub fn show(&mut self, ui: &mut Ui) -> Option<AssetSpec> {
        ui.heading("Static sprite");
        ui.label("Create one indexed-color image with no animation data.");
        ui.add_space(12.0);
        dimensions::pixel_size(ui, &mut self.width, &mut self.height);
        ui.add_space(16.0);
        ui.button("Create sprite")
            .clicked()
            .then_some(AssetSpec::Sprite(uvec2(self.width, self.height)))
    }
}
