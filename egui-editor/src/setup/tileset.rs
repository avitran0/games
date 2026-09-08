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
        ui.heading("Tileset");
        ui.label("Create a collection of same-sized indexed-color tiles.");
        ui.add_space(12.0);
        dimensions::pixel_size(ui, &mut self.width, &mut self.height);
        ui.add_space(16.0);
        ui.button("Create tileset")
            .clicked()
            .then_some(AssetSpec::Tileset(uvec2(self.width, self.height)))
    }
}
