use crate::document::{AssetDocument, AssetKind};
use api::formats::{
    AnimatedSpriteDocument, FontDocument, SpriteDocument, TilemapDocument, TilesetDocument,
};
use rfd::FileDialog;
use std::path::{Path, PathBuf};

pub struct LoadedAsset {
    pub document: AssetDocument,
    pub path: PathBuf,
    pub linked_tileset_path: Option<PathBuf>,
}

pub fn pick_open_path() -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("Pixel assets", &AssetKind::ALL.map(AssetKind::extension))
        .pick_file()
}

pub fn pick_tileset_path() -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("Tileset", &["pxt"])
        .pick_file()
}

pub fn pick_save_path(kind: AssetKind, default_name: &str) -> Option<PathBuf> {
    FileDialog::new()
        .add_filter(kind.title(), &[kind.extension()])
        .set_file_name(default_name)
        .save_file()
        .map(|path| with_extension(path, kind.extension()))
}

pub fn open() -> Result<Option<LoadedAsset>, String> {
    let Some(path) = pick_open_path() else {
        return Ok(None);
    };
    load(&path)
}

pub fn load(path: &Path) -> Result<Option<LoadedAsset>, String> {
    let bytes = std::fs::read(path)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    let Some(magic) = bytes.get(..4) else {
        return Err("File is too short to contain an asset identifier.".into());
    };

    let kind =
        AssetKind::from_magic(magic).ok_or_else(|| "Unrecognized pixel asset file.".to_string())?;
    let (document, linked_tileset_path) = match kind {
        AssetKind::Sprite => (
            AssetDocument::Sprite(
                SpriteDocument::decode(&bytes).map_err(|error| error.to_string())?,
            ),
            None,
        ),
        AssetKind::AnimatedSprite => {
            let document =
                AnimatedSpriteDocument::decode(&bytes).map_err(|error| error.to_string())?;
            (AssetDocument::AnimatedSprite(document), None)
        }
        AssetKind::Font => (
            AssetDocument::Font(FontDocument::decode(&bytes).map_err(|error| error.to_string())?),
            None,
        ),
        AssetKind::Tileset => (
            AssetDocument::Tileset(
                TilesetDocument::decode(&bytes).map_err(|error| error.to_string())?,
            ),
            None,
        ),
        AssetKind::Tilemap => {
            let map = TilemapDocument::decode(&bytes).map_err(|error| error.to_string())?;
            let Some(tileset_path) = FileDialog::new()
                .add_filter("Matching tileset", &["pxt"])
                .pick_file()
            else {
                return Ok(None);
            };
            let tileset_bytes = std::fs::read(&tileset_path).map_err(|error| {
                format!("Could not read tileset {}: {error}", tileset_path.display())
            })?;
            let tileset =
                TilesetDocument::decode(&tileset_bytes).map_err(|error| error.to_string())?;
            if tileset.id != map.tileset_id {
                return Err(format!(
                    "The selected tileset does not match this map (expected {}).",
                    map.tileset_id
                ));
            }
            (AssetDocument::Tilemap { map, tileset }, Some(tileset_path))
        }
    };

    Ok(Some(LoadedAsset {
        document,
        path: path.to_path_buf(),
        linked_tileset_path,
    }))
}

pub fn save(path: &Path, document: &AssetDocument) -> Result<(), String> {
    write_bytes(path, &document.encode()?)
}

pub fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), String> {
    std::fs::write(path, bytes)
        .map_err(|error| format!("Could not save {}: {error}", path.display()))
}

pub fn load_tileset(path: &Path) -> Result<TilesetDocument, String> {
    let bytes = std::fs::read(path)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    TilesetDocument::decode(&bytes).map_err(|error| error.to_string())
}

pub fn file_name(path: Option<&Path>) -> &str {
    path.and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .unwrap_or("Untitled")
}

fn with_extension(mut path: PathBuf, extension: &str) -> PathBuf {
    path.set_extension(extension);
    path
}
