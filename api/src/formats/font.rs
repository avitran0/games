use std::collections::HashSet;

use utils::io::{Endian, EndianReader, ReadBytes};
#[cfg(feature = "dev")]
use utils::io::{EndianWriter, WriteBytes};

use crate::formats::error::FontDecodeError;
#[cfg(feature = "dev")]
use crate::formats::error::FontEncodeError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlyphDocument {
    pub codepoint: char,
    pub width: u16,
    pub advance: u16,
    /// Row-major bitmap. Each pixel uses one byte.
    pub bitmap: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontDocument {
    pub height: u16,
    pub glyphs: Vec<GlyphDocument>,
}

#[cfg(feature = "dev")]
impl Default for FontDocument {
    fn default() -> Self {
        Self::new(11).expect("The 11-pixel font height is valid.")
    }
}

impl FontDocument {
    const MAGIC: [u8; 4] = *b"FONT";
    const VERSION: u16 = 1;

    #[cfg(feature = "dev")]
    pub fn new(height: u16) -> Result<Self, FontEncodeError> {
        validate_font_height(height).map_err(FontEncodeError::InvalidHeight)?;
        let bitmap = vec![0; height as usize * height as usize];
        Ok(Self {
            height,
            glyphs: vec![GlyphDocument {
                codepoint: '\u{FFFD}',
                width: height,
                advance: 1,
                bitmap,
            }],
        })
    }

    #[cfg(feature = "dev")]
    pub fn validate(&self) -> Result<(), FontEncodeError> {
        validate_font_height(self.height).map_err(FontEncodeError::InvalidHeight)?;
        if !(1..=u16::MAX as usize).contains(&self.glyphs.len()) {
            return Err(FontEncodeError::InvalidGlyphCount(self.glyphs.len()));
        }
        let mut codepoints = HashSet::new();
        for glyph in &self.glyphs {
            if !codepoints.insert(glyph.codepoint) {
                return Err(FontEncodeError::DuplicateCodepoint(glyph.codepoint));
            }
            if glyph.width == 0 {
                return Err(FontEncodeError::InvalidGlyphWidth {
                    codepoint: glyph.codepoint,
                    width: glyph.width,
                });
            }
            let expected = usize::from(glyph.width) * usize::from(self.height);
            if glyph.bitmap.len() != expected {
                return Err(FontEncodeError::InvalidBitmapLength {
                    codepoint: glyph.codepoint,
                    expected,
                    actual: glyph.bitmap.len(),
                });
            }
        }
        Ok(())
    }

    #[cfg(feature = "dev")]
    pub fn encode(&self) -> Result<Vec<u8>, FontEncodeError> {
        self.validate()?;
        let mut bytes = Vec::new();
        let mut writer = EndianWriter::new(&mut bytes, Endian::Little);
        writer.write_bytes(&Self::MAGIC)?;
        writer.write_u16(Self::VERSION)?;
        writer.write_u16(self.height)?;
        writer.write_u16(self.glyphs.len() as u16)?;
        for glyph in &self.glyphs {
            writer.write_u32(glyph.codepoint as u32)?;
            writer.write_u16(glyph.advance)?;
            writer.write_u16(glyph.width)?;
            writer.write_bytes(&glyph.bitmap)?;
        }
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, FontDecodeError> {
        let mut reader = EndianReader::new(bytes, Endian::Little);
        let magic = ReadBytes::read_array::<4>(&mut reader)?;
        if magic != Self::MAGIC {
            return Err(FontDecodeError::InvalidMagic(magic));
        }
        let version = reader.read_u16()?;
        if version != Self::VERSION {
            return Err(FontDecodeError::InvalidVersion(version));
        }
        let height = reader.read_u16()?;
        validate_font_height(height).map_err(FontDecodeError::InvalidHeight)?;
        let count = reader.read_u16()? as usize;
        if count == 0 {
            return Err(FontDecodeError::InvalidGlyphCount(count));
        }
        let mut expected_len = 10_usize;
        let mut codepoints = HashSet::new();
        let mut glyphs = Vec::with_capacity(count);
        for _ in 0..count {
            let codepoint = decode_codepoint(reader.read_u32()?)?;
            let advance = reader.read_u16()?;
            let width = reader.read_u16()?;
            if width == 0 {
                return Err(FontDecodeError::InvalidGlyphWidth { codepoint, width });
            }
            let bitmap_len = usize::from(width) * usize::from(height);
            expected_len = expected_len.saturating_add(8 + bitmap_len);
            let bitmap = reader.read_bytes(bitmap_len)?;
            if !codepoints.insert(codepoint) {
                return Err(FontDecodeError::DuplicateCodepoint(codepoint));
            }
            glyphs.push(GlyphDocument {
                codepoint,
                width,
                advance,
                bitmap,
            });
        }
        validate_font_data_length(bytes, expected_len)?;
        Ok(Self { height, glyphs })
    }
}

fn validate_font_data_length(bytes: &[u8], expected: usize) -> Result<(), FontDecodeError> {
    if bytes.len() == expected {
        Ok(())
    } else {
        Err(FontDecodeError::InvalidDataLength {
            expected,
            actual: bytes.len(),
        })
    }
}

fn validate_font_height(height: u16) -> Result<(), u16> {
    if (1..=64).contains(&height) {
        Ok(())
    } else {
        Err(height)
    }
}

fn decode_codepoint(value: u32) -> Result<char, FontDecodeError> {
    char::from_u32(value).ok_or(FontDecodeError::InvalidCodepoint(value))
}

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
