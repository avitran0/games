use api::strum::EnumIter;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, EnumIter)]
#[strum(crate = "api::strum")]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}

impl Difficulty {
    pub const fn mine_density_percent(self) -> u64 {
        match self {
            Self::Easy => 12,
            Self::Normal => 15,
            Self::Hard => 20,
        }
    }
}
