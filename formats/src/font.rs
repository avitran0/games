use std::collections::HashSet;

use utils::io::{Endian, EndianReader, ReadBytes};
#[cfg(feature = "edit")]
use utils::io::{EndianWriter, WriteBytes};

use crate::error::FontDecodeError;
#[cfg(feature = "edit")]
use crate::error::FontEncodeError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlyphDocument {
    pub codepoint: char,
    pub width: u16,
    pub advance: u16,
    /// store one byte per pixel in row-major order.
    pub bitmap: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontDocument {
    pub height: u16,
    pub glyphs: Vec<GlyphDocument>,
}

#[cfg(feature = "edit")]
impl Default for FontDocument {
    fn default() -> Self {
        Self::new(11).expect("The 11-pixel font height is valid.")
    }
}

impl FontDocument {
    const MAGIC: [u8; 4] = *b"FONT";
    const VERSION: u16 = 1;

    #[cfg(feature = "edit")]
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

    #[cfg(feature = "edit")]
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

    #[cfg(feature = "edit")]
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
        Ok(Self { height, glyphs })
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
