#[expect(
    dead_code,
    reason = "larger footprints are reserved for future block kinds"
)]
#[derive(Clone, Copy)]
pub enum BlockSize {
    /// 1x1
    Single,
    /// 2x2
    Double,
    /// 3x3
    Triple,
    /// 4x4
    Quad,
}

impl BlockSize {
    pub const fn cells(self) -> u32 {
        match self {
            Self::Single => 1,
            Self::Double => 2,
            Self::Triple => 3,
            Self::Quad => 4,
        }
    }
}
