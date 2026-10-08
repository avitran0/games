use formats::{SpriteFrame, TilesetDocument};
use glam::U16Vec2;
use uuid::Uuid;

#[derive(Clone)]
pub struct Tileset {
    asset_id: Uuid,
    id: u32,
    name: String,
    tile_count: u32,
    tile_size: U16Vec2,
    tiles: Vec<SpriteFrame>,
    empty_tile_id: u32,
}

impl Tileset {
    pub(crate) fn from_document(document: TilesetDocument) -> Self {
        let asset_id = document.id;
        let mut tiles = vec![SpriteFrame::blank(document.tile_size)];
        let tile_count = document.tiles.len() as u32 + 1;
        tiles.extend(document.tiles);
        Self {
            asset_id,
            id: 0,
            name: "tiles".into(),
            tile_count,
            tile_size: document.tile_size.as_u16vec2(),
            tiles,
            empty_tile_id: 0,
        }
    }

    pub fn asset_id(&self) -> Uuid {
        self.asset_id
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tile_count(&self) -> u32 {
        self.tile_count
    }

    pub fn tile_size(&self) -> U16Vec2 {
        self.tile_size
    }

    /// returns indexed-color pixels in tile id order.
    pub fn tiles(&self) -> impl ExactSizeIterator<Item = &[u8]> {
        self.tiles.iter().map(SpriteFrame::pixels)
    }

    pub fn empty_tile_id(&self) -> u32 {
        self.empty_tile_id
    }
}
