pub(crate) mod color;
pub(crate) mod error;
pub(crate) mod font;
pub(crate) mod palette;
pub(crate) mod sprite;
pub(crate) mod tilemap;
pub(crate) mod tileset;

pub use tileset::Tileset;

#[cfg(feature = "dev")]
pub use error::{
    AnimatedSpriteDecodeError, AnimatedSpriteEncodeError, FontDecodeError, FontEncodeError,
    InvalidFrameError, InvalidSizeError, SpriteDecodeError, SpriteEncodeError, TilemapDecodeError,
    TilemapEncodeError, TilesetDecodeError, TilesetEncodeError,
};
#[cfg(feature = "dev")]
pub use font::{FontDocument, GlyphDocument};
#[cfg(feature = "dev")]
pub use sprite::{
    AnimatedSpriteDocument, AnimationDirection, SpriteDocument, SpriteFrame, SpriteTag,
};
#[cfg(feature = "dev")]
pub use tilemap::{Tile, TilemapDocument};
#[cfg(feature = "dev")]
pub use tileset::TilesetDocument;
