use crate::formats::color::Color;

pub(crate) fn data() -> Vec<u8> {
    Color::ALL.iter().flat_map(|color| color.rgb()).collect()
}
