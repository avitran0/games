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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OreKind {
    Iron,
    Coal,
    Titanium,
}

impl OreKind {
    pub fn item(self) -> crate::core::item::Item {
        match self {
            Self::Iron => crate::core::item::Item::IronOre,
            Self::Coal => crate::core::item::Item::Coal,
            Self::Titanium => crate::core::item::Item::TitaniumOre,
        }
    }
}
