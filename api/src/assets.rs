use std::{collections::HashMap, hash::Hash, rc::Rc};

use formats::{AnimatedSpriteDocument, SpriteDocument, TilemapDocument, TilesetDocument};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    AnimatedSprite, Font, ScreenAction, Sprite,
    formats::{tilemap::TilemapData, tileset::Tileset},
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

    /// load a tileset before loading maps that refer to it.
    pub fn load_tileset<State>(&mut self, data: &[u8]) -> Result<Tileset, ScreenAction<State>> {
        let document = TilesetDocument::decode(data)
            .map_err(|error| show_asset_error(AssetLoadError::Decode(error.to_string())))?;
        let uuid = document.id;
        let tileset = Tileset::from_document(document.clone());
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
