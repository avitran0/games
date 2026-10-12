use api::Color;
use eframe::egui::Color32;

pub const COLOR_COUNT: usize = Color::COUNT;

pub fn color(index: u8) -> Color32 {
    let [red, green, blue] = Color::ALL[usize::from(index)].rgb();
    Color32::from_rgb(red, green, blue)
}
