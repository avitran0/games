/// Maximum width and height of an individual static sprite, animation frame,
/// tileset tile, or sprite-sheet entry, in pixels.
pub const MAX_SPRITE_WIDTH: u32 = 96;
pub const MAX_SPRITE_HEIGHT: u32 = 96;

/// Maximum tilemap dimensions, in cells.
pub const MAX_TILEMAP_WIDTH: u32 = 256;
pub const MAX_TILEMAP_HEIGHT: u32 = 256;

/// Maximum bitmap-font glyph dimensions, in pixels.
pub const MAX_FONT_GLYPH_WIDTH: u16 = u16::MAX;
pub const MAX_FONT_HEIGHT: u16 = 64;

mod error;
mod font;
mod sprite;
mod sprite_sheet;
mod tilemap;
mod tileset;

pub use error::{
    AnimatedSpriteDecodeError, FontDecodeError, InvalidAnimationError, InvalidFrameError,
    InvalidSizeError, SpriteDecodeError, SpriteSheetDecodeError, TilemapDecodeError,
    TilesetDecodeError,
};
#[cfg(feature = "edit")]
pub use error::{
    AnimatedSpriteEncodeError, FontEncodeError, SpriteEncodeError, SpriteSheetEncodeError,
    TilemapEncodeError, TilesetEncodeError,
};
pub use font::{FontDocument, GlyphDocument};
pub use sprite::{
    AnimatedSpriteDocument, AnimationDirection, SpriteDocument, SpriteFrame, SpriteTag,
};
pub use sprite_sheet::{NamedSprite, SpriteSheetDocument};
pub use tilemap::{Tile, TilemapDocument};
pub use tileset::TilesetDocument;
