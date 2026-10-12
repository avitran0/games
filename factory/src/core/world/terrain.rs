#[derive(Clone, Copy, Default)]
pub struct TerrainCell {
    pub ground: GroundKind,
    pub ore: Option<OreKind>,
}

#[derive(Clone, Copy, Default)]
pub enum GroundKind {
    #[default]
    Sand,
    Water,
}

#[derive(Clone, Copy)]
pub enum OreKind {
    Iron,
    Coal,
    Titanium,
}
