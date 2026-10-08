use glam::IVec2;

use crate::Rect;

#[derive(Clone, Copy)]
pub(crate) struct Layout {
    pub bounds: Rect,
    pub cursor: IVec2,
}

#[derive(Clone, Copy)]
pub(crate) struct LayoutSnapshot(pub Layout);

pub(crate) struct Columns {
    pub parent: Layout,
    pub count: usize,
    pub index: usize,
    pub gap: u32,
    pub start_y: i32,
    pub max_bottom: i32,
}
