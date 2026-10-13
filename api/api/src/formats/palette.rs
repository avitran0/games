use crate::formats::color::Color;

#[derive(Clone, Default)]
pub(crate) struct Palette;

impl Palette {
    pub fn data(&self) -> Vec<u8> {
        Color::ALL.iter().flat_map(|color| color.rgb()).collect()
    }
}
