use std::collections::HashSet;

#[cfg(feature = "edit")]
use glam::UVec2;
use glam::uvec2;
use utils::io::{Endian, EndianReader, ReadBytes};
#[cfg(feature = "edit")]
use utils::io::{EndianWriter, WriteBytes};

#[cfg(feature = "edit")]
use crate::error::SpriteSheetEncodeError;
use crate::{
    error::SpriteSheetDecodeError,
    sprite::{validate_frame, validate_sprite_size},
};

use super::{SpriteDocument, SpriteFrame};

/// A named static sprite in a sprite sheet. Each sprite keeps its own dimensions.
#[derive(Clone, PartialEq, Eq)]
pub struct NamedSprite {
    pub name: String,
    pub sprite: SpriteDocument,
}

/// A collection of named, independent static sprites (not a packed texture atlas).
#[derive(Clone, PartialEq, Eq)]
pub struct SpriteSheetDocument {
    pub sprites: Vec<NamedSprite>,
}

impl SpriteSheetDocument {
    const MAGIC: [u8; 4] = *b"SPRS";
    const VERSION: u16 = 1;

    #[cfg(feature = "edit")]
    pub fn new(size: UVec2) -> Result<Self, crate::error::InvalidSizeError> {
        Ok(Self {
            sprites: vec![NamedSprite {
                name: "sprite".into(),
                sprite: SpriteDocument::new(size)?,
            }],
        })
    }

    #[cfg(feature = "edit")]
    pub fn validate(&self) -> Result<(), SpriteSheetEncodeError> {
        if self.sprites.is_empty() || self.sprites.len() > u16::MAX as usize {
            return Err(SpriteSheetEncodeError::InvalidSpriteCount(
                self.sprites.len(),
            ));
        }
        let mut names = HashSet::new();
        for (index, named) in self.sprites.iter().enumerate() {
            if named.name.trim().is_empty() {
                return Err(SpriteSheetEncodeError::EmptyName { sprite: index });
            }
            if named.name.len() > u16::MAX as usize {
                return Err(SpriteSheetEncodeError::NameTooLong {
                    sprite: index,
                    length: named.name.len(),
                });
            }
            if !names.insert(&named.name) {
                return Err(SpriteSheetEncodeError::DuplicateName(named.name.clone()));
            }
            validate_sprite_size(named.sprite.size).map_err(|error| {
                SpriteSheetEncodeError::InvalidSpriteSize {
                    sprite: index,
                    error,
                }
            })?;
            validate_frame(named.sprite.size, &named.sprite.pixels).map_err(|error| {
                SpriteSheetEncodeError::InvalidSpriteFrame {
                    sprite: index,
                    error,
                }
            })?;
        }
        Ok(())
    }

    #[cfg(feature = "edit")]
    pub fn encode(&self) -> Result<Vec<u8>, SpriteSheetEncodeError> {
        self.validate()?;
        let mut buf = Vec::new();
        let mut writer = EndianWriter::new(&mut buf, Endian::Little);
        writer.write_bytes(&Self::MAGIC)?;
        writer.write_u16(Self::VERSION)?;
        writer.write_u16(self.sprites.len() as u16)?;
        for named in &self.sprites {
            writer.write_u16(named.name.len() as u16)?;
            writer.write_bytes(named.name.as_bytes())?;
            writer.write_u16(named.sprite.size.x as u16)?;
            writer.write_u16(named.sprite.size.y as u16)?;
            named.sprite.pixels.encode(&mut writer)?;
        }
        Ok(buf)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, SpriteSheetDecodeError> {
        let mut reader = EndianReader::new(bytes, Endian::Little);
        let magic = ReadBytes::read_array::<4>(&mut reader)?;
        if magic != Self::MAGIC {
            return Err(SpriteSheetDecodeError::InvalidMagic(magic));
        }
        let version = reader.read_u16()?;
        if version != Self::VERSION {
            return Err(SpriteSheetDecodeError::InvalidVersion(version));
        }
        let count = reader.read_u16()? as usize;
        if count == 0 {
            return Err(SpriteSheetDecodeError::InvalidSpriteCount(count));
        }
        let mut sprites = Vec::with_capacity(count);
        let mut names = HashSet::new();
        for index in 0..count {
            let name_len = reader.read_u16()? as usize;
            if name_len == 0 {
                return Err(SpriteSheetDecodeError::EmptyName { sprite: index });
            }
            let name = String::from_utf8(reader.read_bytes(name_len)?)?;
            if name.trim().is_empty() {
                return Err(SpriteSheetDecodeError::EmptyName { sprite: index });
            }
            if !names.insert(name.clone()) {
                return Err(SpriteSheetDecodeError::DuplicateName(name));
            }
            let size = uvec2(reader.read_u16()? as u32, reader.read_u16()? as u32);
            validate_sprite_size(size).map_err(|error| {
                SpriteSheetDecodeError::InvalidSpriteSize {
                    sprite: index,
                    error,
                }
            })?;
            let pixels = SpriteFrame::decode(&mut reader, size)?;
            validate_frame(size, &pixels).map_err(|error| {
                SpriteSheetDecodeError::InvalidSpriteFrame {
                    sprite: index,
                    error,
                }
            })?;
            sprites.push(NamedSprite {
                name,
                sprite: SpriteDocument { size, pixels },
            });
        }
        Ok(Self { sprites })
    }
}

#[cfg(all(test, feature = "edit"))]
mod tests {
    use glam::uvec2;

    use super::{NamedSprite, SpriteSheetDocument};
    use crate::SpriteDocument;

    #[test]
    fn sprite_sheet_round_trips_named_sprites() {
        let mut sheet = SpriteSheetDocument::new(uvec2(8, 8)).unwrap();
        sheet.sprites.push(NamedSprite {
            name: "hero idle".into(),
            sprite: SpriteDocument::new(uvec2(16, 8)).unwrap(),
        });
        let decoded = SpriteSheetDocument::decode(&sheet.encode().unwrap()).unwrap();
        assert!(decoded == sheet);
    }

    #[test]
    fn duplicate_sprite_names_are_rejected() {
        let mut sheet = SpriteSheetDocument::new(uvec2(8, 8)).unwrap();
        sheet.sprites.push(sheet.sprites[0].clone());
        assert!(sheet.encode().is_err());
    }
}
