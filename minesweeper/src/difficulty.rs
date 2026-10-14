use api::glam::{UVec2, uvec2};
use ui::EnumIter;

#[derive(Clone, Copy, Default, Debug, PartialEq, EnumIter)]
#[strum(crate = "ui")]
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

    pub const fn tile_scale(self) -> f32 {
        self.tile_size() as f32 / 16.0
    }

    pub const fn size(self) -> UVec2 {
        uvec2(
            api::WIDTH / self.tile_size(),
            api::HEIGHT / self.tile_size(),
        )
    }
}
