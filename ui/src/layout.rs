use glam::IVec2;

use crate::Rect;

#[derive(Clone, Copy)]
pub(crate) struct Layout {
    pub bounds: Rect,
    pub cursor: IVec2,
}

#[derive(Clone, Copy)]
pub(crate) struct LayoutSnapshot(pub Layout);
