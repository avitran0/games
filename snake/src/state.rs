#[derive(Default)]
pub struct State {
    pub difficulty: Difficulty,
    pub score: u32,
}

#[derive(Default, Clone, Copy, Debug)]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}

impl Difficulty {
    pub fn lower(&mut self) {
        *self = match *self {
            Self::Easy | Self::Normal => Self::Easy,
            Self::Hard => Self::Normal,
        };
    }

    pub fn higher(&mut self) {
        *self = match *self {
            Self::Easy => Self::Normal,
            Self::Normal | Self::Hard => Self::Hard,
        };
    }

    pub fn move_interval(&self) -> usize {
        match self {
            Self::Easy => 25,
            Self::Normal => 15,
            Self::Hard => 12,
        }
    }
}
