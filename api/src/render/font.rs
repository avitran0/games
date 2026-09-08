use std::{collections::HashMap, rc::Rc};

use glam::uvec2;

use crate::{
    formats::font::{Font, Glyph},
    render::gpu_image::GpuImage,
};

pub(crate) struct GlFont {
    image: GpuImage,
    font: Font,
    chars: HashMap<char, usize>,
}

impl GlFont {
    pub(crate) fn new(gl: Rc<glow::Context>, data: &[u8]) -> Result<Self, String> {
        let font = Font::read(data).map_err(|err| err.to_string())?;
        let width = font
            .glyphs
            .iter()
            .map(|glyph| glyph.width as usize)
            .max()
            .unwrap_or(1);
        let height = font.height as usize;
        let mut data = vec![0u8; width * height * font.glyphs.len()];
        let mut chars = HashMap::new();

        for (layer, glyph) in font.glyphs.iter().enumerate() {
            let layer_offset = width * height * layer;
            let stride = (glyph.width as usize).div_ceil(8);

            for y in 0..height {
                for x in 0..glyph.width as usize {
                    let byte = glyph.bitmap[y * stride + x / 8];
                    let bit = (byte >> (7 - (x % 8))) & 1;
                    data[layer_offset + y * width + x] = bit;
                }
            }
            chars.insert(glyph.codepoint, layer);
        }

        let image = GpuImage::new_array(
            gl,
            &data,
            uvec2(width as u32, font.height as u32),
            font.glyphs.len(),
        )?;

        Ok(Self { image, font, chars })
    }

    pub(crate) fn height(&self) -> u16 {
        self.font.height
    }

    pub(crate) fn bind(&self) {
        self.image.bind();
    }

    pub(crate) fn glyph(&self, c: char) -> (&Glyph, usize) {
        let index = self
            .chars
            .get(&c)
            .copied()
            .or_else(|| self.chars.get(&'\u{FFFD}').copied())
            .unwrap_or(0);
        (&self.font.glyphs[index], index)
    }
}
