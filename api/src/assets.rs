use std::{collections::HashMap, hash::Hash, rc::Rc};

use formats::{AnimatedSpriteDocument, SpriteDocument, TilemapDocument, TilesetDocument};
use glam::{U16Vec2, UVec2};
use thiserror::Error;
use utils::info;
use uuid::Uuid;

use crate::{
    AnimatedSprite, Font, ScreenAction, Sprite, Tileset,
    formats::tilemap::TilemapData,
    handle::{SpriteId, Tilemap as TilemapHandle, TilemapId},
    render::{font::GlFont, sprite::GlSprite},
    screen::error::ErrorScreen,
};

struct AssetRegistry<Id: Hash + Eq + Copy, Asset> {
    registry: HashMap<Id, Asset>,
    cache: HashMap<blake3::Hash, Id>,
}

impl<Id: Hash + Eq + Copy, Asset> AssetRegistry<Id, Asset> {
    fn new() -> Self {
        Self {
            registry: HashMap::new(),
            cache: HashMap::new(),
        }
    }

    fn get_cached(&self, hash: blake3::Hash) -> Option<Id> {
        self.cache.get(&hash).copied()
    }

    fn get(&self, id: &Id) -> Option<&Asset> {
        self.registry.get(id)
    }

    fn get_mut(&mut self, id: &Id) -> Option<&mut Asset> {
        self.registry.get_mut(id)
    }

    fn insert_cached(&mut self, id: Id, asset: Asset, hash: blake3::Hash) {
        self.registry.insert(id, asset);
        self.cache.insert(hash, id);
    }

    fn len(&self) -> usize {
        self.registry.len()
    }
}

pub struct Assets {
    gl: Rc<glow::Context>,
    sprites: AssetRegistry<SpriteId, GlSprite>,
    fonts: AssetRegistry<Font, GlFont>,
    tilemaps: AssetRegistry<TilemapId, TilemapData>,
    tileset_sprites: HashMap<Uuid, SpriteId>,
    default_font: Font,
}

impl Assets {
    pub(crate) fn new(gl: Rc<glow::Context>) -> Result<Self, String> {
        let mut fonts = AssetRegistry::new();
        let default_font =
            Self::load_font_into(&gl, &mut fonts, include_bytes!("../assets/font.pxf"))
                .map_err(|err| err.to_string())?;
        info!("initialized assets");

        Ok(Self {
            gl,
            sprites: AssetRegistry::new(),
            fonts,
            tilemaps: AssetRegistry::new(),
            tileset_sprites: HashMap::new(),
            default_font,
        })
    }

    /// use this method for `.pxs` files. use `load_animated_sprite` for `.pxa` files.
    pub fn load_sprite<State>(&mut self, data: &[u8]) -> Result<Sprite, ScreenAction<State>> {
        let document = SpriteDocument::decode(data)
            .map_err(|error| show_asset_error(AssetLoadError::Decode(error.to_string())))?;
        self.register_sprite(data, |gl| GlSprite::new_static(gl, document))
            .map(Sprite::new)
            .map_err(show_asset_error)
    }

    /// each returned handle has separate playback state.
    pub fn load_animated_sprite<State>(
        &mut self,
        data: &[u8],
    ) -> Result<AnimatedSprite, ScreenAction<State>> {
        let document = AnimatedSpriteDocument::decode(data)
            .map_err(|error| show_asset_error(AssetLoadError::Decode(error.to_string())))?;
        self.register_sprite(data, |gl| GlSprite::new_animated(gl, document))
            .map(AnimatedSprite::new)
            .map_err(show_asset_error)
    }

    fn register_sprite(
        &mut self,
        data: &[u8],
        build: impl FnOnce(Rc<glow::Context>) -> Result<GlSprite, String>,
    ) -> Result<SpriteId, AssetLoadError> {
        let hash = blake3::hash(data);
        if let Some(id) = self.sprites.get_cached(hash) {
            return Ok(id);
        }
        let id = SpriteId(
            u16::try_from(self.sprites.len()).map_err(|_| AssetLoadError::TooMany("sprites"))?,
        );
        let sprite = build(self.gl.clone()).map_err(AssetLoadError::Build)?;
        self.sprites.insert_cached(id, sprite, hash);
        Ok(id)
    }

    /// the matching tileset must be loaded first.
    pub fn load_tilemap<State>(
        &mut self,
        data: &[u8],
    ) -> Result<TilemapHandle, ScreenAction<State>> {
        let document = TilemapDocument::decode(data)
            .map_err(|error| show_asset_error(AssetLoadError::Decode(error.to_string())))?;
        if !self.tileset_sprites.contains_key(&document.tileset_id) {
            return Err(show_asset_error(AssetLoadError::MissingTileset(
                document.tileset_id,
            )));
        }
        let hash = blake3::hash(data);
        if let Some(id) = self.tilemaps.get_cached(hash) {
            return Ok(TilemapHandle::new(id));
        }
        let id = TilemapId(
            u16::try_from(self.tilemaps.len())
                .map_err(|_| show_asset_error(AssetLoadError::TooMany("tilemaps")))?,
        );
        self.tilemaps
            .insert_cached(id, TilemapData::from_document(document), hash);
        Ok(TilemapHandle::new(id))
    }

    /// position uses zero-based cell coordinates. tile id 0 clears the cell and flip flags.
    pub fn set_tile(
        &mut self,
        tilemap: &TilemapHandle,
        position: UVec2,
        tile_id: u32,
    ) -> Result<(), TilemapSetError> {
        let map = self
            .tilemaps
            .get(&tilemap.id)
            .ok_or(TilemapSetError::TilemapNotLoaded)?;
        let size = map.size();
        validate_tile_position(position, size)?;
        if tile_id != 0 {
            let tileset_id = map.tileset_id();
            let sprite_id = self
                .tileset_sprites
                .get(&tileset_id)
                .ok_or(TilemapSetError::TilesetNotLoaded(tileset_id))?;
            let tile_count = self
                .sprites
                .get(sprite_id)
                .ok_or(TilemapSetError::TilesetNotLoaded(tileset_id))?
                .frame_count() as u32;
            validate_tile_id(tile_id, tile_count)?;
        }

        self.tilemaps
            .get_mut(&tilemap.id)
            .ok_or(TilemapSetError::TilemapNotLoaded)?
            .set_tile(position, tile_id);
        Ok(())
    }

    /// load a tileset before loading maps that refer to it.
    pub fn load_tileset<State>(&mut self, data: &[u8]) -> Result<Tileset, ScreenAction<State>> {
        let document = TilesetDocument::decode(data)
            .map_err(|error| show_asset_error(AssetLoadError::Decode(error.to_string())))?;
        let uuid = document.id;
        let tileset = Tileset::from_document(&document);
        let sprite = self
            .register_sprite(data, |gl| GlSprite::new_tileset(gl, document))
            .map_err(show_asset_error)?;
        self.tileset_sprites.insert(uuid, sprite);
        Ok(tileset)
    }

    pub fn load_font<State>(&mut self, data: &[u8]) -> Result<Font, ScreenAction<State>> {
        Self::load_font_into(&self.gl, &mut self.fonts, data).map_err(show_asset_error)
    }

    fn load_font_into(
        gl: &Rc<glow::Context>,
        fonts: &mut AssetRegistry<Font, GlFont>,
        data: &[u8],
    ) -> Result<Font, AssetLoadError> {
        let hash = blake3::hash(data);
        if let Some(font) = fonts.get_cached(hash) {
            return Ok(font);
        }

        let font = GlFont::new(gl.clone(), data).map_err(AssetLoadError::Build)?;
        let id = Font(u16::try_from(fonts.len()).map_err(|_| AssetLoadError::TooMany("fonts"))?);
        fonts.insert_cached(id, font, hash);
        Ok(id)
    }

    pub(crate) fn get_sprite(&self, sprite: SpriteId) -> Option<&GlSprite> {
        self.sprites.get(&sprite)
    }

    pub(crate) fn get_font(&self, font: Font) -> Option<&GlFont> {
        self.fonts.get(&font)
    }

    pub(crate) fn get_tilemap(&self, tilemap: TilemapHandle) -> Option<&TilemapData> {
        self.tilemaps.get(&tilemap.id)
    }

    pub(crate) fn tileset_sprite(&self, uuid: Uuid) -> Option<SpriteId> {
        self.tileset_sprites.get(&uuid).copied()
    }

    pub fn default_font(&self) -> Font {
        self.default_font
    }
}

fn show_asset_error<State>(error: AssetLoadError) -> ScreenAction<State> {
    ScreenAction::Push(Box::new(ErrorScreen::new(error)))
}

fn validate_tile_position(position: UVec2, size: U16Vec2) -> Result<(), TilemapSetError> {
    if position.x < u32::from(size.x) && position.y < u32::from(size.y) {
        Ok(())
    } else {
        Err(TilemapSetError::PositionOutOfBounds {
            x: position.x,
            y: position.y,
            width: u32::from(size.x),
            height: u32::from(size.y),
        })
    }
}

fn validate_tile_id(tile_id: u32, tile_count: u32) -> Result<(), TilemapSetError> {
    if tile_id == 0 || tile_id < tile_count {
        Ok(())
    } else {
        Err(TilemapSetError::TileIdOutOfBounds {
            tile_id,
            tile_count,
        })
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TilemapSetError {
    #[error("tilemap is not loaded")]
    TilemapNotLoaded,
    #[error("cell ({x}, {y}) is outside tilemap size {width}x{height}")]
    PositionOutOfBounds {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
    #[error("tileset {0} is not loaded")]
    TilesetNotLoaded(Uuid),
    #[error("tile id {tile_id} is out of range; tileset has {tile_count} tile slots")]
    TileIdOutOfBounds { tile_id: u32, tile_count: u32 },
}

#[derive(Debug, Error)]
pub(crate) enum AssetLoadError {
    #[error("Cannot decode document: {0}")]
    Decode(String),
    #[error("Cannot build GPU asset: {0}")]
    Build(String),
    #[error("The number of {0} is too high.")]
    TooMany(&'static str),
    #[error("No tileset is loaded with UUID {0}.")]
    MissingTileset(Uuid),
}

#[cfg(test)]
mod tests {
    use glam::{u16vec2, uvec2};

    use super::{TilemapSetError, validate_tile_id, validate_tile_position};

    #[test]
    fn validates_tile_positions_and_ids() {
        assert!(validate_tile_position(uvec2(2, 1), u16vec2(3, 2)).is_ok());
        assert!(matches!(
            validate_tile_position(uvec2(3, 0), u16vec2(3, 2)),
            Err(TilemapSetError::PositionOutOfBounds { .. })
        ));
        assert!(validate_tile_id(0, 0).is_ok());
        assert!(validate_tile_id(1, 2).is_ok());
        assert!(matches!(
            validate_tile_id(2, 2),
            Err(TilemapSetError::TileIdOutOfBounds { .. })
        ));
    }
}
