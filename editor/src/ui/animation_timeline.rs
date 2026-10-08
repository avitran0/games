use crate::palette;
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use formats::{AnimatedSpriteDocument, AnimationDirection, SpriteTag};

#[derive(Default)]
pub struct State {
    frame_index: usize,
    tag_index: usize,
    tag_drag: Option<TagEdgeDrag>,
}

#[derive(Clone, Copy)]
struct TagEdgeDrag {
    tag_index: usize,
    edge: TagEdge,
}

#[derive(Clone, Copy)]
enum TagEdge {
    Start,
    End,
}

impl State {
    pub fn frame_index(&self) -> usize {
        self.frame_index
    }

    pub fn select_frame(&mut self, index: usize) {
        self.frame_index = index;
    }

    pub fn tag_index(&self) -> usize {
        self.tag_index
    }

    pub fn select_tag(&mut self, index: usize) {
        self.tag_index = index;
    }

    pub fn clamp_to(&mut self, document: &AnimatedSpriteDocument) {
        self.frame_index = self
            .frame_index
            .min(document.frames.len().saturating_sub(1));
        self.tag_index = self.tag_index.min(document.tags.len().saturating_sub(1));
    }
}

pub fn show(ui: &mut Ui, document: &mut AnimatedSpriteDocument, state: &mut State) -> bool {
    if document.frames.is_empty() {
        ui.label("Animation has no frames.");
        return false;
    }

    let mut changed = false;
    state.clamp_to(document);
    ui.horizontal(|ui| {
        ui.strong("Timeline");
        ui.separator();
        ui.label(format!("{} frames", document.frames.len()));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_enabled(!document.tags.is_empty(), egui::Button::new("Remove tag"))
                .clicked()
            {
                remove_tag(document, state);
                changed = true;
            }
            if ui
                .add_enabled(
                    document.tags.len() < u16::MAX as usize,
                    egui::Button::new("Add tag"),
                )
                .clicked()
            {
                add_tag(document, state);
                changed = true;
            }
        });
    });

    egui::ScrollArea::both()
        .id_salt("animation-timeline-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            const FRAME_WIDTH: f32 = 68.0;
            const FRAME_HEIGHT: f32 = 72.0;
            const TAG_HEIGHT: f32 = 23.0;
            let width = document.frames.len() as f32 * FRAME_WIDTH;
            let height = FRAME_HEIGHT + document.tags.len() as f32 * TAG_HEIGHT;
            let (rect, response) = ui.allocate_exact_size(
                Vec2::new(width.max(ui.available_width()), height.max(FRAME_HEIGHT)),
                Sense::click_and_drag(),
            );
            let painter = ui.painter_at(rect);
            let clip_rect = painter.clip_rect();

            for (index, frame) in document.frames.iter().enumerate() {
                let x = rect.left() + index as f32 * FRAME_WIDTH;
                let frame_rect = Rect::from_min_max(
                    Pos2::new(x, rect.top()),
                    Pos2::new(x + FRAME_WIDTH, rect.top() + FRAME_HEIGHT),
                );
                if !frame_rect.intersects(clip_rect) {
                    continue;
                }
                painter.rect_filled(frame_rect, 0.0, ui.visuals().faint_bg_color);
                let image_rect =
                    Rect::from_min_size(Pos2::new(x + 10.0, rect.top() + 4.0), Vec2::splat(44.0));
                paint_frame_thumbnail(&painter, image_rect, frame.pixels(), document.size);
                painter.text(
                    Pos2::new(x + FRAME_WIDTH / 2.0, rect.top() + 61.0),
                    egui::Align2::CENTER_CENTER,
                    (index + 1).to_string(),
                    egui::FontId::proportional(12.0),
                    ui.visuals().text_color(),
                );
                let stroke = if index == state.frame_index {
                    ui.visuals().selection.stroke
                } else {
                    Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color)
                };
                painter.rect_stroke(frame_rect, 1.0, stroke, egui::StrokeKind::Inside);
            }

            for (index, tag) in document.tags.iter().enumerate() {
                let y = rect.top() + FRAME_HEIGHT + index as f32 * TAG_HEIGHT;
                let row =
                    Rect::from_min_size(Pos2::new(rect.left(), y), Vec2::new(width, TAG_HEIGHT));
                if !row.intersects(clip_rect) {
                    continue;
                }
                painter.rect_filled(row, 0.0, ui.visuals().extreme_bg_color);
                let start_x = rect.left() + tag.start as f32 * FRAME_WIDTH + 3.0;
                let end_x = rect.left() + (tag.end as f32 + 1.0) * FRAME_WIDTH - 3.0;
                let range = Rect::from_min_max(
                    Pos2::new(start_x, y + 2.0),
                    Pos2::new(end_x.max(start_x + 4.0), y + TAG_HEIGHT - 2.0),
                );
                painter.rect_filled(range, 3.0, tag_color(index));
                if index == state.tag_index {
                    painter.rect_stroke(
                        range,
                        3.0,
                        Stroke::new(2.0, ui.visuals().selection.stroke.color),
                        egui::StrokeKind::Inside,
                    );
                    for edge_x in [range.left(), range.right()] {
                        let handle = Rect::from_center_size(
                            Pos2::new(edge_x, range.center().y),
                            Vec2::new(5.0, 12.0),
                        );
                        painter.rect_filled(handle, 1.0, Color32::WHITE);
                        painter.rect_stroke(
                            handle,
                            1.0,
                            Stroke::new(1.0, Color32::DARK_GRAY),
                            egui::StrokeKind::Inside,
                        );
                    }
                }
                painter.text(
                    range.left_center() + Vec2::new(5.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    format!(
                        "{}  {}-{}",
                        tag.name,
                        tag.start.saturating_add(1),
                        tag.end.saturating_add(1)
                    ),
                    egui::FontId::proportional(12.0),
                    Color32::WHITE,
                );
            }

            if response.drag_started_by(egui::PointerButton::Primary)
                && let Some(pointer) = response.interact_pointer_pos()
                && pointer.y >= rect.top() + FRAME_HEIGHT
            {
                let tag_index =
                    ((pointer.y - rect.top() - FRAME_HEIGHT) / TAG_HEIGHT).floor() as usize;
                if let Some(tag) = document.tags.get(tag_index) {
                    state.tag_index = tag_index;
                    let start_x = rect.left() + tag.start as f32 * FRAME_WIDTH + 3.0;
                    let end_x = rect.left() + (tag.end as f32 + 1.0) * FRAME_WIDTH - 3.0;
                    let from_start = (pointer.x - start_x).abs();
                    let from_end = (pointer.x - end_x).abs();
                    state.tag_drag = if from_start <= 10.0 && from_start <= from_end {
                        Some(TagEdgeDrag {
                            tag_index,
                            edge: TagEdge::Start,
                        })
                    } else if from_end <= 10.0 {
                        Some(TagEdgeDrag {
                            tag_index,
                            edge: TagEdge::End,
                        })
                    } else {
                        None
                    };
                }
            }

            if response.dragged_by(egui::PointerButton::Primary)
                && let (Some(drag), Some(pointer)) =
                    (state.tag_drag, response.interact_pointer_pos())
                && let Some(tag) = document.tags.get_mut(drag.tag_index)
            {
                let frame = ((pointer.x - rect.left()) / FRAME_WIDTH)
                    .floor()
                    .clamp(0.0, document.frames.len().saturating_sub(1) as f32)
                    as u16;
                match drag.edge {
                    TagEdge::Start => tag.start = frame.min(tag.end),
                    TagEdge::End => tag.end = frame.max(tag.start),
                }
                changed = true;
            }

            if response.drag_stopped_by(egui::PointerButton::Primary) {
                state.tag_drag = None;
            }

            if response.clicked_by(egui::PointerButton::Primary)
                && let Some(pointer) = response.interact_pointer_pos()
            {
                if pointer.y < rect.top() + FRAME_HEIGHT {
                    state.frame_index = ((pointer.x - rect.left()) / FRAME_WIDTH)
                        .floor()
                        .clamp(0.0, document.frames.len().saturating_sub(1) as f32)
                        as usize;
                } else {
                    let tag_index =
                        ((pointer.y - rect.top() - FRAME_HEIGHT) / TAG_HEIGHT).floor() as usize;
                    if tag_index < document.tags.len() {
                        state.tag_index = tag_index;
                    }
                }
            }
        });
    changed
}

pub fn add_tag(document: &mut AnimatedSpriteDocument, state: &mut State) {
    let mut suffix = document.tags.len() + 1;
    let name = loop {
        let candidate = format!("tag_{suffix}");
        if !document.tags.iter().any(|tag| tag.name == candidate) {
            break candidate;
        }
        suffix += 1;
    };
    let current = state.frame_index.min(document.frames.len() - 1) as u16;
    document.tags.push(SpriteTag {
        name,
        start: current,
        end: current,
        direction: AnimationDirection::Forward,
    });
    state.tag_index = document.tags.len() - 1;
}

pub fn remove_tag(document: &mut AnimatedSpriteDocument, state: &mut State) {
    if document.tags.is_empty() {
        return;
    }
    document
        .tags
        .remove(state.tag_index.min(document.tags.len() - 1));
    state.tag_index = state.tag_index.min(document.tags.len().saturating_sub(1));
}

pub fn duplicate_frame(document: &mut AnimatedSpriteDocument, state: &mut State) {
    let index = state.frame_index.min(document.frames.len() - 1);
    let insert_at = index + 1;
    document
        .frames
        .insert(insert_at, document.frames[index].clone());
    for tag in &mut document.tags {
        if tag.start as usize > index {
            tag.start += 1;
            tag.end += 1;
        } else if tag.end as usize >= index {
            tag.end += 1;
        }
    }
    state.frame_index = insert_at;
}

pub fn remove_frame(document: &mut AnimatedSpriteDocument, state: &mut State) {
    if document.frames.len() <= 1 {
        return;
    }
    let index = state.frame_index.min(document.frames.len() - 1);
    document.frames.remove(index);
    let removed = index as u16;
    document.tags.retain_mut(|tag| {
        if tag.start > removed {
            tag.start -= 1;
            tag.end -= 1;
        } else if tag.end >= removed {
            if tag.start == removed && tag.end == removed {
                return false;
            }
            tag.end -= 1;
        }
        true
    });
    state.frame_index = state.frame_index.min(document.frames.len() - 1);
    state.tag_index = state.tag_index.min(document.tags.len().saturating_sub(1));
}

fn paint_frame_thumbnail(
    painter: &egui::Painter,
    rect: Rect,
    pixels: &[u8],
    size: api::glam::UVec2,
) {
    let (light, dark) = (Color32::from_gray(230), Color32::from_gray(190));
    const CHECKER_SIZE: f32 = 8.0;
    for y in 0..6 {
        for x in 0..6 {
            painter.rect_filled(
                Rect::from_min_size(
                    Pos2::new(
                        rect.left() + x as f32 * CHECKER_SIZE,
                        rect.top() + y as f32 * CHECKER_SIZE,
                    ),
                    Vec2::splat(CHECKER_SIZE),
                ),
                0.0,
                if (x + y) % 2 == 0 { light } else { dark },
            );
        }
    }

    let pixel_size = 44.0 / size.x.max(size.y).max(1) as f32;
    for y in 0..size.y as usize {
        for x in 0..size.x as usize {
            let value = pixels.get(y * size.x as usize + x).copied().unwrap_or(0);
            if value != 0 {
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(
                            rect.left() + x as f32 * pixel_size,
                            rect.top() + y as f32 * pixel_size,
                        ),
                        Vec2::splat(pixel_size),
                    ),
                    0.0,
                    palette::color(value),
                );
            }
        }
    }
}

fn tag_color(index: usize) -> Color32 {
    const COLORS: [[u8; 3]; 8] = [
        [57, 112, 174],
        [72, 139, 96],
        [161, 103, 54],
        [137, 82, 157],
        [171, 76, 85],
        [54, 135, 143],
        [145, 123, 48],
        [84, 102, 151],
    ];
    let [r, g, b] = COLORS[index % COLORS.len()];
    Color32::from_rgb(r, g, b)
}
