use glam::{U16Vec2, UVec2, u16vec2};
use utils::io::{Endian, EndianReader, ReadBytes};
#[cfg(feature = "dev")]
use utils::io::{EndianWriter, WriteBytes};
use uuid::Uuid;

use super::error::TilemapDecodeError;
#[cfg(feature = "dev")]
use super::error::TilemapEncodeError;

pub(crate) struct TilemapData {
    size: U16Vec2,
    tileset_id: Uuid,
    cells: Vec<Tile>,
}

impl TilemapData {
    pub(crate) fn from_document(document: TilemapDocument) -> Self {
        Self {
            size: document.size.as_u16vec2(),
            tileset_id: document.tileset_id,
            cells: document.cells,
        }
    }

    pub(crate) fn size(&self) -> U16Vec2 {
        self.size
    }

    pub(crate) fn tileset_id(&self) -> Uuid {
        self.tileset_id
    }

    pub(crate) fn cells(&self) -> &[Tile] {
        &self.cells
    }
}

/// Stores one layer and one frame in `TMAP` version 1 (`.pxm`).
/// The map refers to a separately stored tileset by its UUID.
#[derive(Clone, PartialEq, Eq)]
pub struct TilemapDocument {
    /// Sets the map width and height in cells. Each value must be from 1 to 256.
    pub size: UVec2,
    pub tileset_id: Uuid,
    /// Stores cells in row-major order. Zero means empty; nonzero IDs select tiles in the referenced tileset.
    pub cells: Vec<Tile>,
}

impl TilemapDocument {
    const MAGIC: [u8; 4] = *b"TMAP";
    const VERSION: u16 = 1;

    #[cfg(feature = "dev")]
    pub fn new(size: UVec2, tileset_id: Uuid) -> Result<Self, TilemapEncodeError> {
        validate_map_size_for_encode(size)?;
        Ok(Self {
            size,
            tileset_id,
            cells: vec![Tile::default(); (size.x * size.y) as usize],
        })
    }

    #[cfg(feature = "dev")]
    pub fn validate(&self) -> Result<(), TilemapEncodeError> {
        validate_map_size_for_encode(self.size)?;
        let expected = (self.size.x * self.size.y) as usize;
        if self.cells.len() != expected {
            return Err(TilemapEncodeError::InvalidCellCount {
                expected,
                actual: self.cells.len(),
            });
        }
        for tile in &self.cells {
            if tile.id > u16::MAX as u32 {
                return Err(TilemapEncodeError::TileIdTooLarge(tile.id));
            }
        }
        Ok(())
    }

    /// Writes the referenced tileset UUID, then fixed-size cells. Each cell has a `u16` ID and `u8` flags.
    #[cfg(feature = "dev")]
    pub fn encode(&self) -> Result<Vec<u8>, TilemapEncodeError> {
        self.validate()?;
        let mut bytes = Vec::new();
        let mut writer = EndianWriter::new(&mut bytes, Endian::Little);
        writer.write_bytes(&Self::MAGIC)?;
        writer.write_u16(Self::VERSION)?;
        writer.write_u16(self.size.x as u16)?;
        writer.write_u16(self.size.y as u16)?;
        writer.write_bytes(self.tileset_id.as_bytes())?;
        for tile in &self.cells {
            writer.write_u16(tile.id as u16)?;
            writer.write_u8(
                u8::from(tile.flip_x)
                    | (u8::from(tile.flip_y) << 1)
                    | (u8::from(tile.flip_diagonal) << 2),
            )?;
        }
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, TilemapDecodeError> {
        let mut input = bytes;
        let mut reader = EndianReader::new(&mut input, Endian::Little);
        let magic = ReadBytes::read_array::<4>(&mut reader)?;
        if magic != Self::MAGIC {
            return Err(TilemapDecodeError::InvalidMagic(magic));
        }
        let version = reader.read_u16()?;
        if version != Self::VERSION {
            return Err(TilemapDecodeError::InvalidVersion(version));
        }
        let size = u16vec2(reader.read_u16()?, reader.read_u16()?).as_uvec2();
        validate_map_size_for_decode(size)?;
        let tileset_id = Uuid::from_bytes(ReadBytes::read_array::<16>(&mut reader)?);
        let cell_count = (size.x * size.y) as usize;
        let expected_len = 26usize + cell_count * 3;
        if bytes.len() != expected_len {
            return Err(TilemapDecodeError::InvalidDataLength {
                expected: expected_len,
                actual: bytes.len(),
            });
        }
        let mut cells = Vec::with_capacity(cell_count);
        for _ in 0..cell_count {
            let id = reader.read_u16()?;
            let flags = reader.read_u8()?;
            if flags & !7 != 0 {
                return Err(TilemapDecodeError::InvalidFlags(flags));
            }
            cells.push(Tile {
                id: id as u32,
                flip_x: flags & 1 != 0,
                flip_y: flags & 2 != 0,
                flip_diagonal: flags & 4 != 0,
            });
        }
        Ok(Self {
            size,
            tileset_id,
            cells,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tile {
    pub id: u32,
    pub flip_x: bool,
    pub flip_y: bool,
    pub flip_diagonal: bool,
}

#[cfg(feature = "dev")]
fn validate_map_size_for_encode(size: UVec2) -> Result<(), TilemapEncodeError> {
    if map_size_is_valid(size) {
        Ok(())
    } else {
        Err(TilemapEncodeError::InvalidSize {
            width: size.x,
            height: size.y,
        })
    }
}

fn validate_map_size_for_decode(size: UVec2) -> Result<(), TilemapDecodeError> {
    if map_size_is_valid(size) {
        Ok(())
    } else {
        Err(TilemapDecodeError::InvalidSize {
            width: size.x,
            height: size.y,
        })
    }
}

fn map_size_is_valid(size: UVec2) -> bool {
    (1..=256).contains(&size.x) && (1..=256).contains(&size.y)
}
