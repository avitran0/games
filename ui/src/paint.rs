use api::{Anchor, Color, Frame};
use glam::IVec2;

use crate::Rect;

pub(crate) enum PaintCmd {
    Rect {
        rect: Rect,
        fill: Color,
        border: Option<(Color, u32)>,
        radius: u32,
    },
    Text {
        text: String,
        position: IVec2,
        color: Color,
    },
    Line {
        start: IVec2,
        end: IVec2,
        color: Color,
    },
}

#[derive(Default)]
pub(crate) struct PaintList(Vec<PaintCmd>);

impl PaintList {
    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn rect(&mut self, rect: Rect, fill: Color, radius: u32) {
        self.0.push(PaintCmd::Rect {
            rect,
            fill,
            border: None,
            radius,
        });
    }

    pub fn bordered_rect(
        &mut self,
        rect: Rect,
        fill: Color,
        border: Color,
        width: u32,
        radius: u32,
    ) {
        self.0.push(PaintCmd::Rect {
            rect,
            fill,
            border: Some((border, width)),
            radius,
        });
    }

    pub fn text(&mut self, text: impl Into<String>, position: IVec2, color: Color) {
        self.0.push(PaintCmd::Text {
            text: text.into(),
            position,
            color,
        });
    }

    pub fn line(&mut self, start: IVec2, end: IVec2, color: Color) {
        self.0.push(PaintCmd::Line { start, end, color });
    }

    pub fn draw(&self, frame: &mut Frame) {
        for command in &self.0 {
            match command {
                PaintCmd::Rect {
                    rect,
                    fill,
                    border,
                    radius,
                } => match border {
                    Some((color, width)) => frame.bordered_rectangle(
                        rect.position,
                        rect.size,
                        *radius,
                        *fill,
                        *color,
                        *width,
                    ),
                    None if *radius > 0 => {
                        frame.rounded_rectangle(rect.position, rect.size, *radius, *fill)
                    }
                    None => frame.rectangle(rect.position, rect.size, *fill),
                },
                PaintCmd::Text {
                    text,
                    position,
                    color,
                } => frame.text_color(text.clone(), *position, Anchor::TopLeft, *color),
                PaintCmd::Line { start, end, color } => frame.line(*start, *end, *color),
            }
        }
    }
}

pub(crate) fn inset_rect(rect: Rect, inset: u32) -> Rect {
    let inset_i = inset as i32;
    Rect::new(
        rect.position.x + inset_i,
        rect.position.y + inset_i,
        rect.size.x.saturating_sub(inset.saturating_mul(2)),
        rect.size.y.saturating_sub(inset.saturating_mul(2)),
    )
}

pub(crate) fn row_rect(position: IVec2, width: u32, height: u32) -> Rect {
    Rect {
        position,
        size: glam::uvec2(width, height),
    }
}
