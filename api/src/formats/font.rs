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
    pub advance: u16,
    /// row-major, one byte per pixel (wasteful, i know...)
    pub bitmap: Vec<u8>,
}

impl GlyphDocument {
    pub fn width(&self, height: u16) -> u16 {
        (0..height)
            .rev()
            .find(|&x| {
                (0..height).any(|y| {
                    self.bitmap
                        .get(y as usize * height as usize + x as usize)
                        .is_some_and(|&p| p != 0)
                })
            })
            .map_or(0, |x| x + 1)
    }
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
        let bitmap_len = self.height as usize * self.height as usize;
        let mut codepoints = HashSet::new();
        for glyph in &self.glyphs {
            if !codepoints.insert(glyph.codepoint) {
                return Err(FontEncodeError::DuplicateCodepoint(glyph.codepoint));
            }
            if glyph.bitmap.len() != bitmap_len {
                return Err(FontEncodeError::InvalidBitmapLength {
                    codepoint: glyph.codepoint,
                    expected: bitmap_len,
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
        let bitmap_len = height as usize * height as usize;
        let expected_len = 10 + count * (6 + bitmap_len);
        if bytes.len() != expected_len {
            return Err(FontDecodeError::InvalidDataLength {
                expected: expected_len,
                actual: bytes.len(),
            });
        }
        let mut codepoints = HashSet::new();
        let mut glyphs = Vec::with_capacity(count);
        for _ in 0..count {
            let codepoint = reader.read_u32()?;
            let advance = reader.read_u16()?;
            let bitmap = reader.read_bytes(bitmap_len)?;
            let codepoint = decode_codepoint(codepoint)?;
            if !codepoints.insert(codepoint) {
                return Err(FontDecodeError::DuplicateCodepoint(codepoint));
            }
            glyphs.push(GlyphDocument {
                codepoint,
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
                let width = glyph.width(height);
                let stride = (width as usize).div_ceil(8);
                let mut bitmap = vec![0; stride * height as usize];
                for y in 0..height as usize {
                    for x in 0..width as usize {
                        if glyph.bitmap[y * height as usize + x] != 0 {
                            bitmap[y * stride + x / 8] |= 0x80 >> (x % 8);
                        }
                    }
                }
                Glyph {
                    codepoint: glyph.codepoint,
                    advance: glyph.advance,
                    width,
                    bitmap,
                }
            })
            .collect();
        Ok(Self { height, glyphs })
    }
}
