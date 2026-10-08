use glam::IVec2;

use crate::Rect;

#[derive(Clone, Copy)]
pub(crate) struct Layout {
    pub bounds: Rect,
    pub cursor: IVec2,
    pub horizontal: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct LayoutSnapshot(pub Layout);

pub(crate) struct Horizontal {
    pub parent: Layout,
    pub count: usize,
    pub index: usize,
    pub gap: u32,
    pub cell_width: u32,
    pub remainder: u32,
    pub start_x: i32,
    pub start_y: i32,
    pub max_bottom: i32,
}

pub(crate) struct Columns {
    pub parent: Layout,
    pub count: usize,
    pub index: usize,
    pub gap: u32,
    pub start_y: i32,
    pub max_bottom: i32,
}
