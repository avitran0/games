use api::Color;
use eframe::egui::Color32;

pub fn color(index: u8) -> Color32 {
    if index == 0 {
        return Color32::TRANSPARENT;
    }
    let [red, green, blue] = Color::ALL[usize::from(index - 1)].rgb();
    Color32::from_rgb(red, green, blue)
}
