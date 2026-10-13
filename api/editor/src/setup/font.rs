use super::{AssetSpec, dimensions};
use eframe::egui::Ui;

pub struct Screen {
    height: u16,
}

impl Default for Screen {
    fn default() -> Self {
        Self { height: 11 }
    }
}

impl Screen {
    pub fn show(&mut self, ui: &mut Ui) -> Option<AssetSpec> {
        ui.heading("Bitmap font");
        ui.label("Create a font with variable-width glyphs and per-glyph advance.");
        ui.add_space(12.0);
        dimensions::font_height(ui, &mut self.height);
        ui.add_space(16.0);
        ui.button("Create font")
            .clicked()
            .then_some(AssetSpec::Font(self.height))
    }
}
