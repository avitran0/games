use formats::TilesetDocument;
use glam::U16Vec2;
use uuid::Uuid;

#[derive(Clone, Copy)]
pub struct Tileset {
    asset_id: Uuid,
    tile_count: u32,
    tile_size: U16Vec2,
}

impl Tileset {
    pub(crate) fn from_document(document: &TilesetDocument) -> Self {
        Self {
            asset_id: document.id,
            tile_count: document.tiles.len() as u32 + 1,
            tile_size: document.tile_size.as_u16vec2(),
        }
    }

    pub fn asset_id(&self) -> Uuid {
        self.asset_id
    }

    pub fn tile_count(&self) -> u32 {
        self.tile_count
    }

    pub fn tile_size(&self) -> U16Vec2 {
        self.tile_size
    }

    pub fn empty_tile_id(&self) -> u32 {
        0
    }
}
