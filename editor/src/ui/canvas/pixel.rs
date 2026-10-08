use super::{CanvasView, GridBackground, active_pointer, canvas_geometry};
use crate::palette;
use api::glam::UVec2;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Ui, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PixelEdit {
    pub x: usize,
    pub y: usize,
    pub value: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PixelRect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelAction {
    Paint(PixelEdit),
    PickColor(u8),
    MoveSelection { from: PixelRect, to: PixelRect },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PixelTool {
    #[default]
    Draw,
    Select,
}

#[derive(Default)]
pub struct PixelSelection {
    tool: PixelTool,
    rect: Option<PixelRect>,
    drag: Option<SelectionDrag>,
}

#[derive(Clone, Copy)]
enum SelectionDrag {
    Marquee {
        start: (usize, usize),
        current: (usize, usize),
    },
    Move {
        source: PixelRect,
        pointer_start: (usize, usize),
        current: PixelRect,
    },
}

impl PixelSelection {
    pub fn clear(&mut self) {
        self.rect = None;
        self.drag = None;
    }

    pub fn has_selection(&self) -> bool {
        self.rect.is_some()
    }

    fn display_rect(&self) -> Option<PixelRect> {
        match self.drag {
            Some(SelectionDrag::Marquee { start, current }) => Some(rect_between(start, current)),
            Some(SelectionDrag::Move { current, .. }) => Some(current),
            None => self.rect,
        }
    }

    fn moving(&self) -> Option<(PixelRect, PixelRect)> {
        match self.drag {
            Some(SelectionDrag::Move {
                source, current, ..
            }) => Some((source, current)),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelMode {
    Indexed,
    Monochrome,
}

pub fn show_pixels(
    ui: &mut Ui,
    pixels: &[u8],
    size: UVec2,
    selected_value: u8,
    mode: PixelMode,
    view: &mut CanvasView,
) -> Option<PixelAction> {
    show_pixels_inner(ui, pixels, size, selected_value, mode, view, None)
}

pub fn show_pixels_with_selection(
    ui: &mut Ui,
    pixels: &[u8],
    size: UVec2,
    selected_value: u8,
    mode: PixelMode,
    selection: &mut PixelSelection,
    view: &mut CanvasView,
) -> Option<PixelAction> {
    show_pixels_inner(
        ui,
        pixels,
        size,
        selected_value,
        mode,
        view,
        Some(selection),
    )
}

pub fn selection_controls(ui: &mut Ui, selection: &mut PixelSelection) {
    ui.horizontal(|ui| {
        ui.selectable_value(&mut selection.tool, PixelTool::Draw, "Draw");
        ui.selectable_value(&mut selection.tool, PixelTool::Select, "Select");
        if selection.has_selection() && ui.button("Deselect").clicked() {
            selection.clear();
        }
    });
}

fn show_pixels_inner(
    ui: &mut Ui,
    pixels: &[u8],
    size: UVec2,
    selected_value: u8,
    mode: PixelMode,
    view: &mut CanvasView,
    selection: Option<&mut PixelSelection>,
) -> Option<PixelAction> {
    let width = size.x as usize;
    let height = size.y as usize;
    let (viewport, rect, cell, response) = canvas_geometry(ui, size, view);

    let mut selection_action = None;
    let mut selection_outline = None;
    let mut preview_pixels = None;
    let mut tool = PixelTool::Draw;
    if let Some(selection) = selection {
        tool = selection.tool;
        if !ui.ctx().egui_wants_keyboard_input()
            && ui.input(|input| input.key_pressed(egui::Key::Escape))
        {
            selection.clear();
        }
        if mode == PixelMode::Indexed && tool == PixelTool::Select {
            selection_action =
                update_selection(ui, &response, rect, cell, width, height, selection);
        }
        let move_preview = match selection_action {
            Some(PixelAction::MoveSelection { from, to }) => Some((from, to)),
            _ => selection.moving(),
        };
        if let Some((from, to)) = move_preview {
            preview_pixels = Some(moved_pixels(pixels, size, from, to));
        }
        selection_outline = selection.display_rect();
    }

    let display_pixels = preview_pixels.as_deref().unwrap_or(pixels);
    let painter = ui.painter_at(viewport);
    let (light, dark) = view.background.colors();
    let grid_stroke = view.background.grid_stroke();

    for y in 0..height {
        for x in 0..width {
            let value = display_pixels.get(y * width + x).copied().unwrap_or(0);
            let pixel_rect = Rect::from_min_size(
                Pos2::new(rect.left() + x as f32 * cell, rect.top() + y as f32 * cell),
                Vec2::splat(cell),
            );
            if !pixel_rect.intersects(viewport) {
                continue;
            }
            let fill = match (mode, value) {
                (PixelMode::Indexed, value) if value != 0 => palette::color(value),
                (PixelMode::Monochrome, value) if value != 0 => match view.background {
                    GridBackground::Light => Color32::BLACK,
                    GridBackground::Dark => Color32::WHITE,
                },
                _ if (x + y).is_multiple_of(2) => light,
                _ => dark,
            };
            painter.rect_filled(pixel_rect, 0.0, fill);
            painter.rect_stroke(
                pixel_rect,
                0.0,
                Stroke::new(0.5, grid_stroke),
                egui::StrokeKind::Inside,
            );
        }
    }
    painter.rect_stroke(
        rect,
        0.0,
        Stroke::new(1.0, ui.visuals().text_color()),
        egui::StrokeKind::Outside,
    );
    if let Some(selection) = selection_outline {
        let selection_rect = Rect::from_min_size(
            Pos2::new(
                rect.left() + selection.x as f32 * cell,
                rect.top() + selection.y as f32 * cell,
            ),
            Vec2::new(
                selection.width as f32 * cell,
                selection.height as f32 * cell,
            ),
        );
        painter.rect_stroke(
            selection_rect,
            0.0,
            Stroke::new(3.0, Color32::BLACK),
            egui::StrokeKind::Outside,
        );
        painter.rect_stroke(
            selection_rect,
            0.0,
            Stroke::new(1.5, Color32::YELLOW),
            egui::StrokeKind::Inside,
        );
    }

    if let Some(action) = selection_action {
        return Some(action);
    }

    if mode == PixelMode::Indexed
        && tool == PixelTool::Draw
        && response.hovered()
        && !ui.ctx().egui_wants_keyboard_input()
        && ui.input(|input| input.key_pressed(egui::Key::P))
        && let Some(pointer) = response.hover_pos()
        && rect.contains(pointer)
    {
        let x = ((pointer.x - rect.left()) / cell).floor() as usize;
        let y = ((pointer.y - rect.top()) / cell).floor() as usize;
        if x < width && y < height {
            return Some(PixelAction::PickColor(
                pixels.get(y * width + x).copied().unwrap_or(0),
            ));
        }
    }

    (tool == PixelTool::Draw)
        .then(|| active_pointer(&response, ui))
        .flatten()
        .and_then(|(pointer, secondary)| {
            let x = ((pointer.x - rect.left()) / cell).floor() as usize;
            let y = ((pointer.y - rect.top()) / cell).floor() as usize;
            (rect.contains(pointer) && x < width && y < height).then_some(PixelAction::Paint(
                PixelEdit {
                    x,
                    y,
                    value: if secondary { 0 } else { selected_value },
                },
            ))
        })
}

fn update_selection(
    ui: &Ui,
    response: &egui::Response,
    canvas_rect: Rect,
    cell: f32,
    width: usize,
    height: usize,
    selection: &mut PixelSelection,
) -> Option<PixelAction> {
    let space_down = ui.input(|input| input.key_down(egui::Key::Space));
    if !space_down
        && response.drag_started_by(egui::PointerButton::Primary)
        && selection.drag.is_none()
    {
        let press_origin = ui
            .input(|input| input.pointer.press_origin())
            .or_else(|| response.interact_pointer_pos());
        if let Some((x, y)) =
            press_origin.and_then(|pos| cell_at(pos, canvas_rect, cell, width, height, false))
        {
            selection.drag = Some(
                if let Some(rect) = selection.rect.filter(|rect| {
                    x >= rect.x
                        && x < rect.x + rect.width
                        && y >= rect.y
                        && y < rect.y + rect.height
                }) {
                    SelectionDrag::Move {
                        source: rect,
                        pointer_start: (x, y),
                        current: rect,
                    }
                } else {
                    SelectionDrag::Marquee {
                        start: (x, y),
                        current: (x, y),
                    }
                },
            );
        }
    }

    let stopped = response.drag_stopped_by(egui::PointerButton::Primary);
    if selection.drag.is_some()
        && (response.dragged_by(egui::PointerButton::Primary) || stopped)
        && let Some(pointer) = ui.input(|input| {
            input
                .pointer
                .interact_pos()
                .or_else(|| input.pointer.latest_pos())
        })
        && let Some((x, y)) = cell_at(pointer, canvas_rect, cell, width, height, true)
    {
        selection.drag = selection.drag.map(|drag| match drag {
            SelectionDrag::Marquee { start, .. } => SelectionDrag::Marquee {
                start,
                current: (x, y),
            },
            SelectionDrag::Move {
                source,
                pointer_start,
                ..
            } => {
                let max_x = width.saturating_sub(source.width) as isize;
                let max_y = height.saturating_sub(source.height) as isize;
                let new_x = (source.x as isize + x as isize - pointer_start.0 as isize)
                    .clamp(0, max_x) as usize;
                let new_y = (source.y as isize + y as isize - pointer_start.1 as isize)
                    .clamp(0, max_y) as usize;
                SelectionDrag::Move {
                    source,
                    pointer_start,
                    current: PixelRect {
                        x: new_x,
                        y: new_y,
                        ..source
                    },
                }
            }
        });
    }

    if stopped {
        return match selection.drag.take()? {
            SelectionDrag::Marquee { start, current } => {
                selection.rect = Some(rect_between(start, current));
                None
            }
            SelectionDrag::Move {
                source, current, ..
            } => {
                selection.rect = Some(current);
                (source != current).then_some(PixelAction::MoveSelection {
                    from: source,
                    to: current,
                })
            }
        };
    }

    if selection.drag.is_none() && response.clicked_by(egui::PointerButton::Primary) {
        let click_position = response.interact_pointer_pos();
        if let Some((x, y)) =
            click_position.and_then(|pos| cell_at(pos, canvas_rect, cell, width, height, false))
        {
            let already_selected = selection.rect.is_some_and(|rect| {
                x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
            });
            if !already_selected {
                selection.rect = Some(PixelRect {
                    x,
                    y,
                    width: 1,
                    height: 1,
                });
            }
        }
    }
    None
}

fn cell_at(
    pointer: Pos2,
    canvas_rect: Rect,
    cell: f32,
    width: usize,
    height: usize,
    clamp: bool,
) -> Option<(usize, usize)> {
    if width == 0 || height == 0 || (!clamp && !canvas_rect.contains(pointer)) {
        return None;
    }
    let x = ((pointer.x - canvas_rect.left()) / cell)
        .floor()
        .clamp(0.0, (width - 1) as f32) as usize;
    let y = ((pointer.y - canvas_rect.top()) / cell)
        .floor()
        .clamp(0.0, (height - 1) as f32) as usize;
    Some((x, y))
}

fn rect_between(first: (usize, usize), second: (usize, usize)) -> PixelRect {
    let x = first.0.min(second.0);
    let y = first.1.min(second.1);
    PixelRect {
        x,
        y,
        width: first.0.abs_diff(second.0) + 1,
        height: first.1.abs_diff(second.1) + 1,
    }
}

fn moved_pixels(pixels: &[u8], size: UVec2, from: PixelRect, to: PixelRect) -> Vec<u8> {
    let width = size.x as usize;
    let mut output = pixels.to_vec();
    let mut selected = Vec::with_capacity(from.width * from.height);
    for y in 0..from.height {
        let start = (from.y + y) * width + from.x;
        selected.extend_from_slice(&pixels[start..start + from.width]);
    }
    for y in 0..from.height {
        let start = (from.y + y) * width + from.x;
        output[start..start + from.width].fill(0);
    }
    for y in 0..to.height {
        for x in 0..to.width {
            let value = selected[y * from.width + x];
            if value != 0 {
                output[(to.y + y) * width + to.x + x] = value;
            }
        }
    }
    output
}
