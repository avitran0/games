use strum::EnumIter;

use crate::core::block_kind::BlockKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter)]
pub enum BlockCategory {
    Logistics,
    Extraction,
    Production,
    Power,
    Core,
}

impl BlockCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::Logistics => "Logistics",
            Self::Extraction => "Extraction",
            Self::Production => "Production",
            Self::Power => "Power",
            Self::Core => "Core",
        }
    }

    pub fn blocks(self) -> impl Iterator<Item = BlockKind> {
        BlockKind::iter().filter(move |kind| kind.category() == self)
    }
}
