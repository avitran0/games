use glam::{IVec2, UVec2, ivec2, uvec2};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub position: IVec2,
    pub size: UVec2,
}

impl Rect {
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            position: ivec2(x, y),
            size: uvec2(width, height),
        }
    }

    pub fn contains(&self, point: IVec2) -> bool {
        point.x >= self.position.x
            && point.y >= self.position.y
            && point.x < self.position.x + self.size.x as i32
            && point.y < self.position.y + self.size.y as i32
    }
}
