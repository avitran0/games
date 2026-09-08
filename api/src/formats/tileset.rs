use glam::{U16Vec2, UVec2, u16vec2};
use utils::io::{Endian, EndianReader, ReadBytes};
#[cfg(feature = "dev")]
use utils::io::{EndianWriter, WriteBytes};
use uuid::Uuid;

use super::{
    error::TilesetDecodeError,
    sprite::{SpriteFrame, validate_frame, validate_sprite_size},
};
#[cfg(feature = "dev")]
use super::{error::TilesetEncodeError, sprite::SpriteDocument};

/// Stores indexed tiles in `TSET` version 1 (`.pxt`). Tile IDs start at one.
/// Tile ID zero means that a map cell is empty. The tileset has no tile zero.
#[derive(Clone, PartialEq, Eq)]
pub struct TilesetDocument {
    pub id: Uuid,
    pub tile_size: UVec2,
    pub tiles: Vec<SpriteFrame>,
}

impl TilesetDocument {
    const MAGIC: [u8; 4] = *b"TSET";
    const VERSION: u16 = 1;

    #[cfg(feature = "dev")]
    pub fn new(tile_size: UVec2) -> Result<Self, TilesetEncodeError> {
        validate_sprite_size(tile_size)?;
        let sprite = SpriteDocument::new(tile_size)?;
        Ok(Self {
            id: Uuid::new_v4(),
            tile_size,
            tiles: vec![sprite.pixels],
        })
    }

    #[cfg(feature = "dev")]
    pub fn validate(&self) -> Result<(), TilesetEncodeError> {
        validate_sprite_size(self.tile_size)?;
        if self.tiles.is_empty() || self.tiles.len() > u16::MAX as usize {
            return Err(TilesetEncodeError::InvalidTileCount(self.tiles.len()));
        }
        for (tile, pixels) in self.tiles.iter().enumerate() {
            validate_frame(self.tile_size, pixels)
                .map_err(|error| TilesetEncodeError::InvalidTile { tile, error })?;
        }
        Ok(())
    }

    /// Writes a fixed-width little-endian header, then one byte for each tile pixel.
    #[cfg(feature = "dev")]
    pub fn encode(&self) -> Result<Vec<u8>, TilesetEncodeError> {
        self.validate()?;
        let mut buf = Vec::new();
        let mut writer = EndianWriter::new(&mut buf, Endian::Little);
        writer.write_bytes(&Self::MAGIC)?;
        writer.write_u16(Self::VERSION)?;
        writer.write_bytes(self.id.as_bytes())?;
        writer.write_u16(self.tile_size.x as u16)?;
        writer.write_u16(self.tile_size.y as u16)?;
        writer.write_u16(self.tiles.len() as u16)?;
        for tile in &self.tiles {
            tile.encode(&mut writer)?;
        }
        Ok(buf)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, TilesetDecodeError> {
        let mut input = bytes;
        let mut reader = EndianReader::new(&mut input, Endian::Little);
        let magic = ReadBytes::read_array::<4>(&mut reader)?;
        if magic != Self::MAGIC {
            return Err(TilesetDecodeError::InvalidMagic(magic));
        }
        let version = reader.read_u16()?;
        if version != Self::VERSION {
            return Err(TilesetDecodeError::InvalidVersion(version));
        }
        let id = Uuid::from_bytes(ReadBytes::read_array::<16>(&mut reader)?);
        let size = u16vec2(reader.read_u16()?, reader.read_u16()?).as_uvec2();
        validate_sprite_size(size)?;
        let count = reader.read_u16()? as usize;
        if count == 0 {
            return Err(TilesetDecodeError::InvalidTileCount(count));
        }
        let expected_len = 28usize + count * size.x as usize * size.y as usize;
        if bytes.len() != expected_len {
            return Err(TilesetDecodeError::InvalidDataLength {
                expected: expected_len,
                actual: bytes.len(),
            });
        }
        let mut tiles = Vec::with_capacity(count);
        for tile in 0..count {
            let pixels = SpriteFrame::decode(&mut reader, size)?;
            validate_frame(size, &pixels)
                .map_err(|error| TilesetDecodeError::InvalidTile { tile, error })?;
            tiles.push(pixels);
        }
        Ok(Self {
            id,
            tile_size: size,
            tiles,
        })
    }
}

#[derive(Clone)]
pub struct Tileset {
    asset_id: Uuid,
    id: u32,
    name: String,
    tile_count: u32,
    tile_size: U16Vec2,
    tiles: Vec<SpriteFrame>,
    empty_tile_id: u32,
}

impl Tileset {
    pub(crate) fn from_document(document: TilesetDocument) -> Self {
        let asset_id = document.id;
        let mut tiles = vec![SpriteFrame::blank(document.tile_size)];
        let tile_count = document.tiles.len() as u32 + 1;
        tiles.extend(document.tiles);
        Self {
            asset_id,
            id: 0,
            name: "tiles".into(),
            tile_count,
            tile_size: document.tile_size.as_u16vec2(),
            tiles,
            empty_tile_id: 0,
        }
    }

    pub fn asset_id(&self) -> Uuid {
        self.asset_id
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tile_count(&self) -> u32 {
        self.tile_count
    }

    pub fn tile_size(&self) -> U16Vec2 {
        self.tile_size
    }

    /// Returns indexed-color pixels for each tile in ID order.
    pub fn tiles(&self) -> impl ExactSizeIterator<Item = &[u8]> {
        self.tiles.iter().map(SpriteFrame::pixels)
    }

    pub fn empty_tile_id(&self) -> u32 {
        self.empty_tile_id
    }
}
