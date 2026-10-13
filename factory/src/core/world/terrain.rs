#[derive(Clone, Copy, Default)]
pub struct TerrainCell {
    pub ground: GroundKind,
    #[expect(
        dead_code,
        reason = "ore placement is reserved for future world generation"
    )]
    pub ore: Option<OreKind>,
}

#[expect(dead_code, reason = "additional terrain types await world generation")]
#[derive(Clone, Copy, Default)]
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

#[expect(
    dead_code,
    reason = "ore types will be used by terrain generation and drills"
)]
#[derive(Clone, Copy)]
pub enum OreKind {
    Iron,
    Coal,
    Titanium,
}
