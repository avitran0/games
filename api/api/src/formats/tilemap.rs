use formats::{Tile, TilemapDocument};
use glam::{U16Vec2, UVec2};
use uuid::Uuid;

pub(crate) struct TilemapData {
    size: U16Vec2,
    tileset_id: Uuid,
    cells: Vec<Tile>,
}

impl TilemapData {
    pub(crate) fn new(size: U16Vec2, tileset_id: Uuid) -> Self {
        Self {
            size,
            tileset_id,
            cells: vec![Tile::default(); usize::from(size.x) * usize::from(size.y)],
        }
    }

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

    pub(crate) fn set_tile(&mut self, position: UVec2, id: u32) {
        let index = position.y as usize * usize::from(self.size.x) + position.x as usize;
        self.cells[index] = Tile {
            id,
            ..Tile::default()
        };
    }
}
