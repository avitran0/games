#[cfg(feature = "edit")]
use std::io::Write;
use std::{collections::HashSet, io::Read};

use glam::{UVec2, u16vec2};
use utils::io::{Endian, EndianReader, ReadBytes};
#[cfg(feature = "edit")]
use utils::io::{EndianWriter, WriteBytes};

use crate::error::{
    AnimatedSpriteDecodeError, InvalidAnimationError, InvalidFrameError, InvalidSizeError,
    SpriteDecodeError,
};
#[cfg(feature = "edit")]
use crate::error::{AnimatedSpriteEncodeError, SpriteEncodeError};

#[derive(Clone, PartialEq, Eq)]
pub struct SpriteDocument {
    pub size: UVec2,
    pub pixels: SpriteFrame,
}

#[derive(Clone, PartialEq, Eq)]
pub struct AnimatedSpriteDocument {
    pub size: UVec2,
    pub frames: Vec<SpriteFrame>,
    pub tags: Vec<SpriteTag>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct SpriteFrame {
    /// pixels use row-major order. index 0 is transparent.
    pixels: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationDirection {
    Forward,
    Reverse,
    PingPong,
    PingPongReverse,
}

#[derive(Clone, PartialEq, Eq)]
pub struct SpriteTag {
    pub name: String,
    pub start: u16,
    pub end: u16,
    pub direction: AnimationDirection,
}

impl SpriteDocument {
    const MAGIC: [u8; 4] = *b"SPRT";
    const VERSION: u16 = 1;

    #[cfg(feature = "edit")]
    pub fn new(size: UVec2) -> Result<Self, InvalidSizeError> {
        validate_sprite_size(size)?;
        Ok(Self {
            size,
            pixels: SpriteFrame::blank(size),
        })
    }

    #[cfg(feature = "edit")]
    pub fn encode(&self) -> Result<Vec<u8>, SpriteEncodeError> {
        validate_sprite_size(self.size)?;
        validate_frame(self.size, &self.pixels)?;

        let mut buf = Vec::new();
        let mut writer = EndianWriter::new(&mut buf, Endian::Little);

        writer.write_bytes(&Self::MAGIC)?;
        writer.write_u16(Self::VERSION)?;
        writer.write_u16(self.size.x as u16)?;
        writer.write_u16(self.size.y as u16)?;
        self.pixels.encode(&mut writer)?;

        Ok(buf)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, SpriteDecodeError> {
        let mut reader = EndianReader::new(bytes, Endian::Little);

        let magic = ReadBytes::read_array::<4>(&mut reader)?;
        if magic != Self::MAGIC {
            return Err(SpriteDecodeError::InvalidMagic(magic));
        }

        let version = reader.read_u16()?;
        if version != Self::VERSION {
            return Err(SpriteDecodeError::InvalidVersion(version));
        }

        let size = u16vec2(reader.read_u16()?, reader.read_u16()?).as_uvec2();
        validate_sprite_size(size)?;

        let pixels = SpriteFrame::decode(&mut reader, size)?;
        validate_frame(size, &pixels)?;
        Ok(Self { size, pixels })
    }
}

impl AnimatedSpriteDocument {
    const MAGIC: [u8; 4] = *b"ANIM";
    const VERSION: u16 = 1;

    #[cfg(feature = "edit")]
    pub fn new(size: UVec2) -> Result<Self, InvalidSizeError> {
        validate_sprite_size(size)?;
        Ok(Self {
            size,
            frames: vec![SpriteFrame::blank(size)],
            tags: vec![SpriteTag {
                name: "default".into(),
                start: 0,
                end: 0,
                direction: AnimationDirection::Forward,
            }],
        })
    }

    #[cfg(feature = "edit")]
    pub fn validate(&self) -> Result<(), AnimatedSpriteEncodeError> {
        validate_sprite_size(self.size)?;
        validate_animation_frame_count(self.frames.len())?;

        for (i, frame) in self.frames.iter().enumerate() {
            validate_frame(self.size, frame)
                .map_err(|error| AnimatedSpriteEncodeError::InvalidFrame { frame: i, error })?;
        }

        self.validate_tags()?;
        Ok(())
    }

    fn validate_tags(&self) -> Result<(), InvalidAnimationError> {
        if self.tags.len() > u16::MAX as usize {
            return Err(InvalidAnimationError::TooManyTags(self.tags.len()));
        }

        let mut names = HashSet::new();
        for tag in &self.tags {
            if tag.name.trim().is_empty() {
                return Err(InvalidAnimationError::TagNameEmpty);
            }
            if tag.name.len() > u16::MAX as usize {
                return Err(InvalidAnimationError::TagNameTooLong(tag.name.len()));
            }
            if !names.insert(&tag.name) {
                return Err(InvalidAnimationError::TagNameDuplicated(tag.name.clone()));
            }
            if tag.start > tag.end || tag.end as usize >= self.frames.len() {
                return Err(InvalidAnimationError::TagInvalidRange(tag.start..=tag.end));
            }
        }
        Ok(())
    }

    #[cfg(feature = "edit")]
    pub fn encode(&self) -> Result<Vec<u8>, AnimatedSpriteEncodeError> {
        self.validate()?;

        let mut buf = Vec::new();
        let mut writer = EndianWriter::new(&mut buf, Endian::Little);

        writer.write_bytes(&Self::MAGIC)?;
        writer.write_u16(Self::VERSION)?;
        writer.write_u16(self.size.x as u16)?;
        writer.write_u16(self.size.y as u16)?;
        writer.write_u16(self.frames.len() as u16)?;
        writer.write_u16(self.tags.len() as u16)?;

        for frame in &self.frames {
            frame.encode(&mut writer)?;
        }

        for tag in &self.tags {
            writer.write_u16(tag.name.len() as u16)?;
            writer.write_bytes(tag.name.as_bytes())?;
            writer.write_u16(tag.start)?;
            writer.write_u16(tag.end)?;
            writer.write_u8(match tag.direction {
                AnimationDirection::Forward => 0,
                AnimationDirection::Reverse => 1,
                AnimationDirection::PingPong => 2,
                AnimationDirection::PingPongReverse => 3,
            })?;
        }

        Ok(buf)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, AnimatedSpriteDecodeError> {
        let mut reader = EndianReader::new(bytes, Endian::Little);

        let magic = ReadBytes::read_array(&mut reader)?;
        if magic != Self::MAGIC {
            return Err(AnimatedSpriteDecodeError::InvalidMagic(magic));
        }

        let version = reader.read_u16()?;
        if version != Self::VERSION {
            return Err(AnimatedSpriteDecodeError::InvalidVersion(version));
        }

        let size = u16vec2(reader.read_u16()?, reader.read_u16()?).as_uvec2();
        validate_sprite_size(size)?;

        let frame_count = reader.read_u16()? as usize;
        validate_animation_frame_count(frame_count)
            .map_err(AnimatedSpriteDecodeError::InvalidDocument)?;
        let tag_count = reader.read_u16()? as usize;

        let mut frames = Vec::with_capacity(frame_count);
        for i in 0..frame_count {
            let frame = SpriteFrame::decode(&mut reader, size)?;
            validate_frame(size, &frame)
                .map_err(|error| AnimatedSpriteDecodeError::InvalidFrame { frame: i, error })?;
            frames.push(frame);
        }

        let mut tags = Vec::new();
        for _ in 0..tag_count {
            let name_len = reader.read_u16()? as usize;
            if name_len == 0 {
                return Err(AnimatedSpriteDecodeError::EmptyTagName);
            }
            let name = String::from_utf8(reader.read_bytes(name_len)?)?;

            let start = reader.read_u16()?;
            let end = reader.read_u16()?;
            let direction = match reader.read_u8()? {
                0 => AnimationDirection::Forward,
                1 => AnimationDirection::Reverse,
                2 => AnimationDirection::PingPong,
                3 => AnimationDirection::PingPongReverse,
                value => {
                    utils::warn!("invalid animation direction: {value}");
                    AnimationDirection::Forward
                }
            };

            tags.push(SpriteTag {
                name,
                start,
                end,
                direction,
            });
        }

        let document = AnimatedSpriteDocument { size, frames, tags };
        document
            .validate_tags()
            .map_err(AnimatedSpriteDecodeError::InvalidDocument)?;
        Ok(document)
    }
}

fn validate_animation_frame_count(count: usize) -> Result<(), InvalidAnimationError> {
    if (1..=u16::MAX as usize).contains(&count) {
        Ok(())
    } else {
        Err(InvalidAnimationError::InvalidFrameCount(count))
    }
}

impl SpriteFrame {
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// validate the document before you encode it after changing pixels.
    #[cfg(feature = "edit")]
    pub fn pixels_mut(&mut self) -> &mut [u8] {
        &mut self.pixels
    }

    pub fn blank(size: UVec2) -> Self {
        Self {
            pixels: vec![0; (size.x * size.y) as usize],
        }
    }

    #[cfg(feature = "edit")]
    pub(crate) fn encode<W: Write>(&self, writer: &mut EndianWriter<W>) -> std::io::Result<()> {
        writer.write_bytes(&self.pixels)
    }

    pub(crate) fn decode<R: Read>(
        reader: &mut EndianReader<R>,
        size: UVec2,
    ) -> std::io::Result<Self> {
        let pixels = reader.read_bytes((size.x * size.y) as usize)?;
        Ok(Self { pixels })
    }
}

pub(super) fn validate_sprite_size(size: UVec2) -> Result<(), InvalidSizeError> {
    if [size.x, size.y]
        .into_iter()
        .all(|side| (8..=96).contains(&side) && side.is_multiple_of(8))
    {
        Ok(())
    } else {
        Err(InvalidSizeError::new(size))
    }
}

pub(super) fn validate_frame(size: UVec2, frame: &SpriteFrame) -> Result<(), InvalidFrameError> {
    if frame.pixels.len() != (size.x * size.y) as usize {
        return Err(InvalidFrameError::BufferLength {
            expected: (size.x * size.y) as usize,
            actual: frame.pixels.len(),
        });
    }
    Ok(())
}

#[cfg(all(test, feature = "edit"))]
mod tests {
    use glam::uvec2;

    use super::{SpriteDocument, validate_sprite_size};

    #[test]
    fn sprite_sizes_allow_up_to_96_pixels() {
        assert!(validate_sprite_size(uvec2(96, 96)).is_ok());
        assert!(validate_sprite_size(uvec2(104, 8)).is_err());
    }

    #[test]
    fn editor_can_mutate_sprite_frame_pixels() {
        let mut sprite = SpriteDocument::new(uvec2(8, 8)).unwrap();
        sprite.pixels.pixels_mut()[0] = 255;
        assert_eq!(sprite.pixels.pixels()[0], 255);
        assert!(sprite.encode().is_ok());
    }
}
