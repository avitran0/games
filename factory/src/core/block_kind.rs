use strum::{EnumIter, IntoEnumIterator};

use crate::core::block::{BlockCategory, BlockSize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumIter)]
pub enum BlockKind {
    Conveyor,
    Drill,
}

impl BlockKind {
    pub fn iter() -> impl Iterator<Item = Self> {
        <Self as IntoEnumIterator>::iter()
    }

    pub fn definition(self) -> BlockDefinition {
        match self {
            Self::Conveyor => BlockDefinition {
                label: "Conveyor",
                category: BlockCategory::Logistics,
                size: BlockSize::Single,
            },
            Self::Drill => BlockDefinition {
                label: "Drill",
                category: BlockCategory::Extraction,
                size: BlockSize::Double,
            },
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
