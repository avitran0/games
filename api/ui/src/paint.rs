use api::{
    Anchor, Color, Frame,
    glam::{IVec2, uvec2},
};

use crate::Rect;

pub(crate) enum PaintCmd {
    Rect {
        rect: Rect,
        fill: Color,
        radius: u32,
    },
    Text {
        text: String,
        position: IVec2,
        color: Color,
        height: u32,
        width: u32,
    },
    Scroll {
        viewport: Rect,
        offset: IVec2,
        commands: Vec<PaintCmd>,
    },
}

#[derive(Default)]
pub(crate) struct PaintList(Vec<PaintCmd>);

impl PaintList {
    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn take_from(&mut self, start: usize) -> Vec<PaintCmd> {
        self.0.split_off(start)
    }

    pub fn scroll(&mut self, viewport: Rect, offset: IVec2, commands: Vec<PaintCmd>) {
        self.0.push(PaintCmd::Scroll {
            viewport,
            offset,
            commands,
        });
    }

    pub fn rect(&mut self, rect: Rect, fill: Color, radius: u32) {
        self.0.push(PaintCmd::Rect { rect, fill, radius });
    }

    pub fn text(
        &mut self,
        text: impl Into<String>,
        position: IVec2,
        color: Color,
        height: u32,
        width: u32,
    ) {
        self.0.push(PaintCmd::Text {
            text: text.into(),
            position,
            color,
            height,
            width,
        });
    }

    pub fn draw(&self, frame: &mut Frame) {
        for command in &self.0 {
            draw_command(frame, command, None, IVec2::ZERO);
        }
    }
}

fn draw_command(frame: &mut Frame, command: &PaintCmd, clip: Option<Rect>, offset: IVec2) {
    match command {
        PaintCmd::Rect { rect, fill, radius } => {
            let mut rect = translated(*rect, offset);
            if let Some(clip) = clip {
                let Some(clipped) = intersection(rect, clip) else {
                    return;
                };
                rect = clipped;
            }
            if *radius > 0 {
                frame.rounded_rectangle(rect.position, rect.size, *radius, *fill);
            } else {
                frame.rectangle(rect.position, rect.size, *fill);
            }
        }
        PaintCmd::Text {
            text,
            position,
            color,
            height,
            width,
        } => {
            let position = *position - offset;
            if let Some(clip) = clip {
                let text_right = position.x.saturating_add(*width as i32);
                let text_bottom = position.y.saturating_add(*height as i32);
                let clip_right = clip.position.x.saturating_add(clip.size.x as i32);
                let clip_bottom = clip.position.y.saturating_add(clip.size.y as i32);
                if position.x < clip.position.x
                    || text_right > clip_right
                    || position.y < clip.position.y
                    || text_bottom > clip_bottom
                {
                    return;
                }
            }
            frame.text_color(text.clone(), position, Anchor::TopLeft, *color);
        }
        PaintCmd::Scroll {
            viewport,
            offset: inner_offset,
            commands,
        } => {
            let viewport = translated(*viewport, offset);
            let visible = match clip {
                Some(clip) => intersection(viewport, clip),
                None => Some(viewport),
            };
            let Some(visible) = visible else {
                return;
            };
            for child in commands {
                draw_command(frame, child, Some(visible), offset + *inner_offset);
            }
        }
    }
}

fn translated(mut rect: Rect, offset: IVec2) -> Rect {
    rect.position -= offset;
    rect
}

fn intersection(a: Rect, b: Rect) -> Option<Rect> {
    let left = a.position.x.max(b.position.x);
    let top = a.position.y.max(b.position.y);
    let right = (a.position.x + a.size.x as i32).min(b.position.x + b.size.x as i32);
    let bottom = (a.position.y + a.size.y as i32).min(b.position.y + b.size.y as i32);
    if right <= left || bottom <= top {
        None
    } else {
        Some(Rect::new(
            left,
            top,
            (right - left) as u32,
            (bottom - top) as u32,
        ))
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
        size: uvec2(width, height),
    }
}
