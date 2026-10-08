use formats::{Tile, TilemapDocument};
use glam::U16Vec2;
use uuid::Uuid;

pub(crate) struct TilemapData {
    size: U16Vec2,
    tileset_id: Uuid,
    cells: Vec<Tile>,
}

impl TilemapData {
    pub(crate) fn from_document(document: TilemapDocument) -> Self {
        Self {
            size: document.size.as_u16vec2(),
            tileset_id: document.tileset_id,
            cells: document.cells,
        }
    }

    pub(crate) fn size(&self) -> U16Vec2 {
        self.size
    }

    pub(crate) fn tileset_id(&self) -> Uuid {
        self.tileset_id
    }

    pub(crate) fn cells(&self) -> &[Tile] {
        &self.cells
    }
}
