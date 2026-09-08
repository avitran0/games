mod app;
mod document;
mod editors;
mod file_io;
mod palette;
mod setup;
mod ui;

use app::App;
use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Pixel Sprite Studio",
        options,
        Box::new(|_creation_context| Ok(Box::<App>::default())),
    )
}
