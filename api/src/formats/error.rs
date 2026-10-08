use std::ops::RangeInclusive;
use std::string::FromUtf8Error;

use glam::UVec2;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Invalid size: {}x{}", size.x, size.y)]
pub struct InvalidSizeError {
    size: UVec2,
}

impl InvalidSizeError {
    pub(crate) fn new(size: UVec2) -> Self {
        Self { size }
    }
}

#[derive(Debug, Error)]
pub enum InvalidFrameError {
    #[error("Invalid buffer length: expected {expected}, got {actual}")]
    BufferLength { expected: usize, actual: usize },
    #[cfg(feature = "dev")]
    #[error("Invalid pixel index: {0}")]
    PixelIndex(u8),
}

#[derive(Debug, Error)]
pub enum SpriteDecodeError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Invalid magic for sprite: {0:X?}")]
    InvalidMagic([u8; 4]),
    #[error("Invalid version: expected 1, got {0}")]
    InvalidVersion(u16),
    #[error(transparent)]
    InvalidSize(#[from] InvalidSizeError),
    #[error(transparent)]
    InvalidFrame(#[from] InvalidFrameError),
}

#[cfg(feature = "dev")]
#[derive(Debug, Error)]
pub enum SpriteEncodeError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    InvalidSize(#[from] InvalidSizeError),
    #[error(transparent)]
    InvalidFrame(#[from] InvalidFrameError),
}

#[derive(Debug, Error)]
pub enum AnimatedSpriteDecodeError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Invalid magic for sprite: {0:X?}")]
    InvalidMagic([u8; 4]),
    #[error("Invalid version: expected 1, got {0}")]
    InvalidVersion(u16),
    #[error(transparent)]
    InvalidSize(#[from] InvalidSizeError),
    #[error("Frame {frame} is invalid: {error}")]
    InvalidFrame {
        frame: usize,
        error: InvalidFrameError,
    },
    #[error("Tag name is empty")]
    EmptyTagName,
    #[error("Invalid tag name: {0}")]
    InvalidTagName(#[from] FromUtf8Error),
    #[error("Invalid animated sprite document: {0}")]
    InvalidDocument(#[source] AnimatedSpriteEncodeError),
}

#[derive(Debug, Error)]
pub enum AnimatedSpriteEncodeError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    InvalidSize(#[from] InvalidSizeError),
    #[cfg(feature = "dev")]
    #[error("Frame {frame} is invalid: {error}")]
    InvalidFrame {
        frame: usize,
        error: InvalidFrameError,
    },
    #[error("Invalid frame count: expected 1..=65535, got {0}")]
    InvalidFrameCount(usize),
    #[error("Sprite has too many tags: max 65535, got {0}")]
    TooManyTags(usize),
    #[error("Tag name is empty")]
    TagNameEmpty,
    #[error("Tag name is too long: {0} bytes")]
    TagNameTooLong(usize),
    #[error("Found duplicate tag name: '{0}'")]
    TagNameDuplicated(String),
    #[error("Invalid tag range: {0:?}")]
    TagInvalidRange(RangeInclusive<u16>),
}

#[cfg(feature = "dev")]
#[derive(Debug, Error)]
pub enum TilesetEncodeError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    InvalidSize(#[from] InvalidSizeError),
    #[error("Invalid tileset tile count: {0}")]
    InvalidTileCount(usize),
    #[error("Tile {tile} is invalid: {error}")]
    InvalidTile {
        tile: usize,
        error: InvalidFrameError,
    },
}

#[derive(Debug, Error)]
pub enum TilesetDecodeError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Invalid magic for tileset: {0:X?}")]
    InvalidMagic([u8; 4]),
    #[error("Invalid version: expected 1, got {0}")]
    InvalidVersion(u16),
    #[error(transparent)]
    InvalidSize(#[from] InvalidSizeError),
    #[error("Invalid tileset tile count: {0}")]
    InvalidTileCount(usize),
    #[error("Tile {tile} is invalid: {error}")]
    InvalidTile {
        tile: usize,
        error: InvalidFrameError,
    },
    #[error("Invalid tileset data length: expected {expected}, got {actual}")]
    InvalidDataLength { expected: usize, actual: usize },
}

#[cfg(feature = "dev")]
#[derive(Debug, Error)]
pub enum TilemapEncodeError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Invalid tilemap dimensions: {width}x{height}")]
    InvalidSize { width: u32, height: u32 },
    #[error("Invalid tilemap cell count: expected {expected}, got {actual}")]
    InvalidCellCount { expected: usize, actual: usize },
    #[error("Tile ID does not fit in the file format: {0}")]
    TileIdTooLarge(u32),
}

#[derive(Debug, Error)]
pub enum TilemapDecodeError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Invalid magic for tilemap: {0:X?}")]
    InvalidMagic([u8; 4]),
    #[error("Invalid version: expected 1, got {0}")]
    InvalidVersion(u16),
    #[error("Invalid tilemap dimensions: {width}x{height}")]
    InvalidSize { width: u32, height: u32 },
    #[error("Invalid tilemap data length: expected {expected}, got {actual}")]
    InvalidDataLength { expected: usize, actual: usize },
    #[error("Invalid tile flip flags: {0:#04x}")]
    InvalidFlags(u8),
}

#[cfg(feature = "dev")]
#[derive(Debug, Error)]
pub enum FontEncodeError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Invalid font height: {0}; expected 1..=64")]
    InvalidHeight(u16),
    #[error("Invalid glyph count: {0}; expected 1..=65535")]
    InvalidGlyphCount(usize),
    #[error("Duplicate glyph codepoint: {0:?}")]
    DuplicateCodepoint(char),
    #[error("Glyph {codepoint:?} bitmap length is invalid: expected {expected}, got {actual}")]
    InvalidBitmapLength {
        codepoint: char,
        expected: usize,
        actual: usize,
    },
}

#[derive(Debug, Error)]
pub enum FontDecodeError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Invalid magic for font: {0:X?}")]
    InvalidMagic([u8; 4]),
    #[error("Invalid font version: expected 1, got {0}")]
    InvalidVersion(u16),
    #[error("Invalid font height: {0}; expected 1..=64")]
    InvalidHeight(u16),
    #[error("Invalid glyph count: {0}; expected 1..=65535")]
    InvalidGlyphCount(usize),
    #[error("Invalid Unicode codepoint: {0:#X}")]
    InvalidCodepoint(u32),
    #[error("Duplicate glyph codepoint: {0:?}")]
    DuplicateCodepoint(char),
    #[error("Invalid font data length: expected {expected}, got {actual}")]
    InvalidDataLength { expected: usize, actual: usize },
}
