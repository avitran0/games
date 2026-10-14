#[derive(Clone, Copy, Default)]
pub struct Tile {
    pub kind: TileKind,
    pub uncovered: bool,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum TileKind {
    #[default]
    Clear,
    Bomb,
}
