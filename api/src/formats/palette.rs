use crate::formats::color::Color;

#[derive(Clone)]
pub(crate) struct Palette {
    colors: [Color; 64],
}

impl Palette {
    pub fn data(&self) -> Vec<u8> {
        self.colors.iter().flat_map(|color| color.rgb()).collect()
    }
}

impl Default for Palette {
    fn default() -> Self {
        let mut colors = [Color::DarkPlum; 64];
        colors[..Color::ALL.len()].copy_from_slice(&Color::ALL);
        Self { colors }
    }
}
