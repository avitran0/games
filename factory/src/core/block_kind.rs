use strum::{EnumIter, IntoEnumIterator};

use crate::core::block::{BlockCategory, BlockSize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumIter)]
pub enum BlockKind {
    Conveyor,
    Storage,
    Drill,
    Smelter,
    SolarPanel,
    Core,
}

impl BlockKind {
    pub fn iter() -> impl Iterator<Item = Self> {
        <Self as IntoEnumIterator>::iter()
    }

    pub fn definition(self) -> BlockDefinition {
        let (label, category, size) = match self {
            Self::Conveyor => ("Conveyor", BlockCategory::Logistics, BlockSize::Single),
            Self::Storage => ("Storage", BlockCategory::Logistics, BlockSize::Double),
            Self::Drill => ("Drill", BlockCategory::Extraction, BlockSize::Double),
            Self::Smelter => ("Smelter", BlockCategory::Production, BlockSize::Double),
            Self::SolarPanel => ("Solar panel", BlockCategory::Power, BlockSize::Single),
            Self::Core => ("Core", BlockCategory::Core, BlockSize::Quad),
        };
        BlockDefinition {
            label,
            category,
            size,
        }
    }

    pub fn category(self) -> BlockCategory {
        self.definition().category
    }

    pub fn label(self) -> &'static str {
        self.definition().label
    }

    pub fn size(self) -> BlockSize {
        self.definition().size
    }
}

#[derive(Clone, Copy)]
pub struct BlockDefinition {
    pub label: &'static str,
    pub category: BlockCategory,
    pub size: BlockSize,
}
