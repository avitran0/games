use api::glam::{UVec2, uvec2};

use crate::sprites::Sprites;

#[derive(Clone, Copy, Default)]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
}

impl Difficulty {
    pub const fn tile_size(self) -> u32 {
        match self {
            Self::Easy => 16,
            Self::Normal => 8,
        }
    }

    pub const fn size(self) -> UVec2 {
        uvec2(
            api::WIDTH / self.tile_size(),
            api::HEIGHT / self.tile_size(),
        )
    }
}
