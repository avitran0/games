use formats::{Tile, TilemapDocument};
use glam::{U16Vec2, UVec2};
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

    pub(crate) fn set_tile(&mut self, position: UVec2, id: u32) {
        let index = position.y as usize * usize::from(self.size.x) + position.x as usize;
        self.cells[index] = Tile {
            id,
            ..Tile::default()
        };
    }
}

#[cfg(test)]
mod tests {
    use formats::{Tile, TilemapDocument};
    use glam::uvec2;
    use uuid::Uuid;

    use super::TilemapData;

    #[test]
    fn set_tile_updates_the_cell_and_clears_flip_flags() {
        let document = TilemapDocument {
            size: uvec2(3, 2),
            tileset_id: Uuid::nil(),
            cells: vec![
                Tile {
                    id: 1,
                    flip_x: true,
                    flip_y: true,
                    flip_diagonal: true,
                };
                6
            ],
        };
        let mut map = TilemapData::from_document(document);
        map.set_tile(uvec2(2, 1), 2);
        assert_eq!(
            map.cells()[5],
            Tile {
                id: 2,
                ..Tile::default()
            }
        );
    }
}
