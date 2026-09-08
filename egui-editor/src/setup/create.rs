use super::AssetSpec;
use crate::{document::AssetDocument, file_io};
use api::formats::{
    AnimatedSpriteDocument, FontDocument, SpriteDocument, TilemapDocument, TilesetDocument,
};
pub fn create(spec: AssetSpec) -> Result<Option<file_io::LoadedAsset>, String> {
    let (document, linked_tileset_path) = match spec {
        AssetSpec::Sprite(size) => (
            AssetDocument::Sprite(SpriteDocument::new(size).map_err(|error| error.to_string())?),
            None,
        ),
        AssetSpec::AnimatedSprite(size) => (
            AssetDocument::AnimatedSprite(
                AnimatedSpriteDocument::new(size).map_err(|error| error.to_string())?,
            ),
            None,
        ),
        AssetSpec::Font(height) => (
            AssetDocument::Font(FontDocument::new(height).map_err(|error| error.to_string())?),
            None,
        ),
        AssetSpec::Tileset(size) => (
            AssetDocument::Tileset(TilesetDocument::new(size).map_err(|error| error.to_string())?),
            None,
        ),
        AssetSpec::Tilemap {
            size,
            tileset,
            tileset_path,
        } => {
            let map = TilemapDocument::new(size, tileset.id).map_err(|error| error.to_string())?;
            (AssetDocument::Tilemap { map, tileset }, Some(tileset_path))
        }
    };

    let kind = document.kind();
    let Some(path) = file_io::pick_save_path(kind, kind.default_filename()) else {
        return Ok(None);
    };
    file_io::save(&path, &document)?;
    Ok(Some(file_io::LoadedAsset {
        document,
        path,
        linked_tileset_path,
    }))
}
