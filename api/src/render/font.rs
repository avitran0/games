use std::{collections::HashMap, rc::Rc, sync::Arc};

use glam::uvec2;

use crate::{
    formats::font::{Font, Glyph},
    render::gpu_image::GpuImage,
};

#[derive(Clone, Debug)]
pub struct TextMetrics {
    advances: Arc<HashMap<char, u32>>,
    fallback_advance: u32,
    line_height: u32,
}

impl TextMetrics {
    pub fn measure_text(&self, text: &str) -> glam::UVec2 {
        let mut widths = text.split('\n').map(|line| {
            line.chars().fold(0_u32, |width, character| {
                width.saturating_add(
                    self.advances
                        .get(&character)
                        .copied()
                        .unwrap_or(self.fallback_advance),
                )
            })
        });
        let width = widths.next().unwrap_or(0).max(widths.max().unwrap_or(0));
        let line_count = text.split('\n').count() as u32;
        glam::uvec2(width, self.line_height.saturating_mul(line_count))
    }
}

pub(crate) struct GlFont {
    image: GpuImage,
    font: Font,
    chars: HashMap<char, usize>,
    text_metrics: TextMetrics,
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use glam::uvec2;

    use super::TextMetrics;

    #[test]
    fn text_metrics_measure_width_and_multiline_height() {
        let metrics = TextMetrics {
            advances: Arc::new(HashMap::from([('a', 2), ('b', 3)])),
            fallback_advance: 4,
            line_height: 7,
        };

        assert_eq!(metrics.measure_text("ab\na"), uvec2(5, 14));
        assert_eq!(metrics.measure_text("a?"), uvec2(6, 7));
    }
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

        let fallback_advance = font
            .glyphs
            .iter()
            .find(|glyph| glyph.codepoint == '\u{FFFD}')
            .or_else(|| font.glyphs.first())
            .map_or(0, |glyph| u32::from(glyph.advance));
        let text_metrics = TextMetrics {
            advances: Arc::new(
                font.glyphs
                    .iter()
                    .map(|glyph| (glyph.codepoint, u32::from(glyph.advance)))
                    .collect(),
            ),
            fallback_advance,
            line_height: u32::from(font.height),
        };
        let image = GpuImage::new_array(
            gl,
            &data,
            uvec2(width as u32, font.height as u32),
            font.glyphs.len(),
        )?;

        Ok(Self {
            image,
            font,
            chars,
            text_metrics,
        })
    }

    pub(crate) fn height(&self) -> u16 {
        self.font.height
    }

    pub(crate) fn text_metrics(&self) -> TextMetrics {
        self.text_metrics.clone()
    }

    pub(crate) fn measure_text(&self, text: &str) -> glam::UVec2 {
        self.text_metrics.measure_text(text)
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
