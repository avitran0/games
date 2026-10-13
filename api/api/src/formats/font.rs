use formats::{FontDecodeError, FontDocument};

#[derive(Debug)]
pub(crate) struct Font {
    pub(crate) height: u16,
    pub(crate) glyphs: Vec<Glyph>,
}

#[derive(Debug)]
pub(crate) struct Glyph {
    pub(crate) codepoint: char,
    pub(crate) advance: u16,
    pub(crate) width: u16,
    pub(crate) bitmap: Vec<u8>,
}

impl Font {
    pub(crate) fn read(data: &[u8]) -> Result<Self, FontDecodeError> {
        let document = FontDocument::decode(data)?;
        let height = document.height;
        let glyphs = document
            .glyphs
            .iter()
            .map(|glyph| {
                let width = usize::from(glyph.width);
                let stride = width.div_ceil(8);
                let mut bitmap = vec![0; stride * usize::from(height)];
                for y in 0..usize::from(height) {
                    for x in 0..width {
                        if glyph.bitmap[y * width + x] != 0 {
                            bitmap[y * stride + x / 8] |= 0x80 >> (x % 8);
                        }
                    }
                }
                Glyph {
                    codepoint: glyph.codepoint,
                    advance: glyph.advance,
                    width: glyph.width,
                    bitmap,
                }
            })
            .collect();
        Ok(Self { height, glyphs })
    }
}
