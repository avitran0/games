use api::Rng;

#[derive(Clone, Copy, Default)]
pub struct TerrainCell {
    pub ground: GroundKind,
    pub ore: Option<OreKind>,
}

#[derive(Clone, Copy, Default)]
pub enum GroundKind {
    #[default]
    Stone,
    Sand,
    Water,
}

impl GroundKind {
    pub fn tile_id(self, x: u32, y: u32) -> u32 {
        let mut rng = Rng::seeded((x as u64) << 32 | y as u64);
        let ids = self.tile_ids_for();
        *rng.choice(ids)
    }

    fn tile_ids_for(self) -> &'static [u32] {
        match self {
            Self::Stone => &[0],
            Self::Sand => &[1],
            Self::Water => &[2],
        }
    }
}

#[derive(Clone, Copy)]
pub enum OreKind {
    Iron,
    Coal,
    Titanium,
}
