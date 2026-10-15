use api::strum::EnumIter;

use crate::config::Config;

#[derive(Default)]
pub struct State {
    pub config: Config,
    pub difficulty: Difficulty,
    pub score: u32,
}

api::impl_app_state!(State);

#[derive(Default, Clone, Copy, Debug, PartialEq, EnumIter)]
#[strum(crate = "api::strum")]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}

impl Difficulty {
    pub fn move_interval(&self) -> usize {
        match self {
            Self::Easy => 50,
            Self::Normal => 44,
            Self::Hard => 40,
        }
    }
}
