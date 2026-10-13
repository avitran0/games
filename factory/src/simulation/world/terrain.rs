use crate::simulation::item::Item;

#[derive(Clone, Copy, Default)]
pub struct TerrainCell {
    pub ground: GroundKind,
    pub ore: Option<OreKind>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GroundKind {
    #[default]
    Stone,
    Sand,
    Water,
}

impl GroundKind {
    pub fn tile_id(self) -> u32 {
        match self {
            Self::Stone => 0,
            Self::Sand => 1,
            Self::Water => 2,
        }
    }
}

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, strum::EnumIter, strum::EnumCount)]
pub enum OreKind {
    Iron,
    Coal,
    Titanium,
}

impl OreKind {
    pub const fn index(self) -> usize {
        self as usize
    }

    pub fn item(self) -> Item {
        match self {
            Self::Iron => Item::IronOre,
            Self::Coal => Item::Coal,
            Self::Titanium => Item::TitaniumOre,
        }
    }
}
