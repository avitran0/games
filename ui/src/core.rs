use api::{Button as EngineButton, Color, Frame, HEIGHT, Input, WIDTH};
use glam::{IVec2, ivec2};

use crate::{
    Rect, Response, Style,
    layout::{Columns, Horizontal, Layout, LayoutSnapshot},
    paint::{PaintList, inset_rect},
    widgets::{Label, Widget},
};

pub struct Ui {
    pub style: Style,
    tick: usize,
    input: Input,
    layout: Layout,
    root_bounds: Rect,
    panels: Vec<LayoutSnapshot>,
    columns: Vec<Columns>,
    horizontals: Vec<Horizontal>,
    paint: PaintList,
    widget_count: usize,
    previous_widget_count: usize,
    focus: usize,
    focused_widget: Option<(usize, Rect)>,
    active_widget: Option<(usize, usize)>,
    activated: bool,
}

impl Default for Ui {
    fn default() -> Self {
        Self::new()
    }
}

impl Ui {
    pub fn new() -> Self {
        let style = Style::default();
        let padding = style.outer_padding;
        let bounds = Rect::new(
            padding as i32,
            padding as i32,
            WIDTH.saturating_sub(padding.saturating_mul(2)),
            HEIGHT.saturating_sub(padding.saturating_mul(2)),
        );
        Self {
            style,
            tick: 0,
            input: Input::default(),
            layout: Layout {
                bounds,
                cursor: bounds.position,
                horizontal: false,
            },
            root_bounds: bounds,
            panels: Vec::new(),
            columns: Vec::new(),
            horizontals: Vec::new(),
            paint: PaintList::default(),
            widget_count: 0,
            previous_widget_count: 0,
            focus: 0,
            focused_widget: None,
            active_widget: None,
            activated: false,
        }
    }

    pub fn begin_frame(&mut self, input: &Input) {
        self.input = *input;
        self.tick += 1;
        self.previous_widget_count = self.widget_count;
        self.widget_count = 0;
        self.paint.clear();
        self.panels.clear();
        self.columns.clear();
        self.horizontals.clear();
        self.focused_widget = None;
        self.layout = Layout {
            bounds: self.root_bounds,
            cursor: self.root_bounds.position,
            horizontal: false,
        };
        if self.previous_widget_count > 0 {
            if input.just_pressed(EngineButton::Up) {
                self.focus =
                    (self.focus + self.previous_widget_count - 1) % self.previous_widget_count;
            } else if input.just_pressed(EngineButton::Down) {
                self.focus = (self.focus + 1) % self.previous_widget_count;
            }
        }
        self.activated =
            input.just_pressed(EngineButton::A) || input.just_pressed(EngineButton::Start);
    }

    pub fn set_bounds(&mut self, bounds: Rect) {
        self.root_bounds = bounds;
        self.layout = Layout {
            bounds,
            cursor: bounds.position,
            horizontal: false,
        };
    }

    pub fn add<W: Widget>(&mut self, widget: W) -> Response {
        widget.show(self)
    }

    pub fn add_contents<R>(&mut self, contents: impl FnOnce(&mut Self) -> R) -> R {
        contents(self)
    }

    pub fn panel<R>(
        &mut self,
        rect: Rect,
        title: impl AsRef<str>,
        contents: impl FnOnce(&mut Self) -> R,
    ) -> R {
        self.begin_panel(rect, title);
        let output = contents(self);
        self.end_panel();
        output
    }

    pub fn columns(&mut self, count: usize, gap: u32, mut contents: impl FnMut(&mut Self, usize)) {
        self.begin_columns(count, gap);
        for index in 0..count.max(1).min(u32::MAX as usize) {
            if index > 0 {
                self.next_column();
            }
            contents(self, index);
        }
        self.end_columns();
    }

    pub fn horizontal(&mut self, count: usize, gap: u32, contents: impl FnOnce(&mut Self)) {
        self.begin_horizontal(count, gap);
        contents(self);
        self.end_horizontal();
    }

    pub fn scroll_area<R>(
        &mut self,
        rect: Rect,
        offset: &mut u32,
        contents: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let parent = self.layout;
        let old_offset = (*offset).min(i32::MAX as u32) as i32;
        self.paint.rect(rect, self.style.panel, 0);
        let paint_start = self.paint.len();
        let widget_start = self.widget_count;
        let start_y = rect.position.y.saturating_sub(old_offset);
        let scrollbar_width = self.style.scrollbar_width.min(rect.size.x);
        let mut content_rect = rect;
        self.layout = Layout {
            bounds: rect,
            cursor: glam::ivec2(rect.position.x, start_y),
            horizontal: false,
        };

        const GAP: u32 = 2;

        let output = contents(self);
        let content_height = self.layout.cursor.y.saturating_sub(start_y).max(0) as u32;
        let max_offset = content_height.saturating_sub(rect.size.y);
        if max_offset > 0 {
            content_rect.size.x = rect.size.x - scrollbar_width - GAP;
        }
        let mut new_offset = old_offset.max(0) as u32;
        if widget_start == self.widget_count {
            if self.input.just_pressed(EngineButton::Up) {
                new_offset = new_offset.saturating_sub(self.style.row_height);
            } else if self.input.just_pressed(EngineButton::Down) {
                new_offset = new_offset.saturating_add(self.style.row_height);
            }
        }
        if let Some((index, focus_rect)) = self.focused_widget
            && index >= widget_start
            && index < self.widget_count
        {
            let target_y =
                rect.position.y + rect.size.y.saturating_sub(focus_rect.size.y) as i32 / 2;
            let adjustment = focus_rect.position.y.saturating_sub(target_y);
            if adjustment >= 0 {
                new_offset = new_offset.saturating_add(adjustment as u32);
            } else {
                new_offset = new_offset.saturating_sub(adjustment.unsigned_abs());
            }
        }
        new_offset = new_offset.min(max_offset);
        let delta = new_offset as i32 - old_offset;
        let commands = self.paint.take_from(paint_start);
        self.paint.scroll(content_rect, delta, commands);
        if max_offset > 0 && scrollbar_width > 0 {
            let track = Rect {
                position: glam::ivec2(
                    rect.position.x + content_rect.size.x as i32 + GAP as i32,
                    rect.position.y,
                ),
                size: glam::uvec2(scrollbar_width, rect.size.y),
            };
            let thumb_height = ((rect.size.y as u64 * rect.size.y as u64)
                / content_height.max(1) as u64)
                .max(4)
                .min(rect.size.y as u64) as u32;
            let travel = rect.size.y - thumb_height;
            let thumb_offset = (new_offset as u64 * travel as u64 / max_offset as u64) as u32;
            let thumb = Rect {
                position: glam::ivec2(track.position.x, track.position.y + thumb_offset as i32),
                size: glam::uvec2(scrollbar_width, thumb_height),
            };
            self.paint.rect(track, self.style.scrollbar_track, 0);
            self.paint.rect(thumb, self.style.scrollbar_thumb, 0);
        }
        self.layout = parent;
        self.advance(rect.size.y + self.style.spacing);
        *offset = new_offset;
        output
    }

    pub fn background(&mut self, rect: Rect, color: Color) {
        self.paint.rect(rect, color, 0);
    }

    fn begin_panel(&mut self, rect: Rect, title: impl AsRef<str>) {
        self.panels.push(LayoutSnapshot(self.layout));
        self.paint
            .rect(rect, self.style.panel, self.style.corner_radius);
        let bounds = inset_rect(rect, self.style.padding);
        self.layout = Layout {
            bounds,
            cursor: bounds.position,
            horizontal: false,
        };
        if !title.as_ref().is_empty() {
            self.add(Label::new(title.as_ref()));
        }
    }

    fn end_panel(&mut self) {
        if let Some(LayoutSnapshot(layout)) = self.panels.pop() {
            self.layout = layout;
        }
    }

    pub fn add_space(&mut self, pixels: u32) {
        self.advance(pixels);
    }

    fn begin_columns(&mut self, count: usize, gap: u32) {
        let count = count.clamp(1, u32::MAX as usize);
        let parent = self.layout;
        self.columns.push(Columns {
            parent,
            count,
            index: 0,
            gap,
            start_y: parent.cursor.y,
            max_bottom: parent.cursor.y,
        });
        self.set_column_layout();
    }

    fn next_column(&mut self) -> bool {
        let Some(columns) = self.columns.last_mut() else {
            return false;
        };
        columns.max_bottom = columns.max_bottom.max(self.layout.cursor.y);
        if columns.index + 1 >= columns.count {
            return false;
        }
        columns.index += 1;
        self.set_column_layout();
        true
    }

    fn end_columns(&mut self) {
        let Some(columns) = self.columns.pop() else {
            return;
        };
        let bottom = columns.max_bottom.max(self.layout.cursor.y);
        self.layout = columns.parent;
        self.layout.cursor.y = bottom;
    }

    fn begin_horizontal(&mut self, count: usize, gap: u32) {
        let count = count.clamp(1, u32::MAX as usize);
        let parent = self.layout;
        let gap_total =
            gap.saturating_mul(u32::try_from(count.saturating_sub(1)).unwrap_or(u32::MAX));
        let available = parent.bounds.size.x.saturating_sub(gap_total);
        self.horizontals.push(Horizontal {
            parent,
            count,
            index: 0,
            gap,
            cell_width: available / count as u32,
            remainder: available % count as u32,
            start_x: parent.cursor.x,
            start_y: parent.cursor.y,
            max_bottom: parent.cursor.y,
        });
        self.set_horizontal_layout();
    }

    fn end_horizontal(&mut self) {
        let Some(horizontal) = self.horizontals.pop() else {
            return;
        };
        let bottom = horizontal.max_bottom.max(self.layout.cursor.y);
        self.layout = horizontal.parent;
        self.layout.cursor.y = bottom;
    }

    fn advance_horizontal_item(&mut self, height: u32) -> bool {
        if !self.layout.horizontal {
            return false;
        }
        let Some(horizontal) = self.horizontals.last_mut() else {
            return false;
        };
        horizontal.max_bottom = horizontal
            .max_bottom
            .max(self.layout.cursor.y.saturating_add(height as i32));
        if horizontal.index + 1 < horizontal.count {
            horizontal.index += 1;
            self.set_horizontal_layout();
        }
        true
    }

    pub fn paint(&self, frame: &mut Frame) {
        self.paint.draw(frame);
    }

    pub(crate) fn input(&self) -> &Input {
        &self.input
    }

    pub(crate) fn reset_focus(&mut self) {
        self.focus = self.widget_count;
        self.focused_widget = None;
    }

    pub(crate) fn widget_response(&mut self) -> Response {
        let focused = self.widget_count == self.focus;
        if focused {
            self.focused_widget = Some((self.widget_count, self.row_rect()));
        }
        let response = Response {
            focused,
            ..Response::default()
        };
        self.widget_count += 1;
        Response {
            clicked: response.focused && self.activated,
            ..response
        }
    }

    fn set_horizontal_layout(&mut self) {
        let Some(horizontal) = self.horizontals.last() else {
            return;
        };
        let parent = horizontal.parent;
        let index = horizontal.index as u32;
        let x = horizontal.start_x
            + index.saturating_mul(horizontal.cell_width.saturating_add(horizontal.gap)) as i32;
        let width = horizontal.cell_width
            + u32::from(index == horizontal.count as u32 - 1) * horizontal.remainder;
        let bottom = parent.bounds.position.y + parent.bounds.size.y as i32;
        let height = bottom.saturating_sub(horizontal.start_y).max(0) as u32;
        self.layout = Layout {
            bounds: Rect::new(x, horizontal.start_y, width, height),
            cursor: glam::ivec2(x, horizontal.start_y),
            horizontal: true,
        };
    }

    fn set_column_layout(&mut self) {
        let Some(columns) = self.columns.last() else {
            return;
        };
        let parent = columns.parent;
        let gap_total = columns
            .gap
            .saturating_mul(u32::try_from(columns.count.saturating_sub(1)).unwrap_or(u32::MAX));
        let available = parent.bounds.size.x.saturating_sub(gap_total);
        let width = available / columns.count as u32;
        let remainder = available % columns.count as u32;
        let index = columns.index as u32;
        let x = parent.bounds.position.x
            + index.saturating_mul(width.saturating_add(columns.gap)) as i32;
        let width = width + u32::from(index == columns.count as u32 - 1) * remainder;
        let bottom = parent.bounds.position.y + parent.bounds.size.y as i32;
        let height = bottom.saturating_sub(columns.start_y).max(0) as u32;
        self.layout = Layout {
            bounds: Rect::new(x, columns.start_y, width, height),
            cursor: glam::ivec2(x, columns.start_y),
            horizontal: false,
        };
    }

    pub(crate) fn row_rect(&self) -> Rect {
        crate::paint::row_rect(
            self.layout.cursor,
            self.layout.bounds.size.x,
            self.style.row_height,
        )
    }

    pub(crate) fn advance_row(&mut self) {
        self.advance(self.style.row_height + self.style.spacing);
    }

    pub(crate) fn advance(&mut self, pixels: u32) {
        if !self.advance_horizontal_item(pixels) {
            self.layout.cursor.y += pixels as i32;
        }
    }

    pub(crate) fn draw_text(&mut self, text: impl Into<String>, position: IVec2, color: Color) {
        self.paint
            .text(text, position, color, self.style.row_height);
    }

    pub(crate) fn draw_rect(&mut self, rect: Rect, color: Color, radius: u32) {
        self.paint.rect(rect, color, radius);
    }

    pub(crate) fn draw_bordered_rect(
        &mut self,
        rect: Rect,
        fill: Color,
        border: Color,
        width: u32,
        radius: u32,
    ) {
        self.paint.bordered_rect(rect, fill, border, width, radius);
    }

    pub(crate) fn draw_interactive(
        &mut self,
        rect: Rect,
        text: &str,
        selected: bool,
        response: Response,
    ) {
        const ACTIVE_TICKS: usize = 15;

        if response.clicked {
            self.active_widget = Some((self.focus, self.tick));
        }
        let active = self.active_widget.is_some_and(|(widget, tick)| {
            widget == self.focus && self.tick.saturating_sub(tick) < ACTIVE_TICKS
        });
        let fill = if response.focused {
            if active {
                self.style.widget_pressed
            } else {
                self.style.widget_focused
            }
        } else if selected {
            self.style.widget_focused
        } else {
            self.style.widget
        };
        let border = if response.focused {
            self.style.text
        } else {
            self.style.panel_border
        };
        self.draw_bordered_rect(
            rect,
            fill,
            border,
            self.style.border_width,
            self.style.corner_radius,
        );
        let prefix = if response.focused { "> " } else { "  " };
        let color = if response.focused || selected {
            self.style.text
        } else {
            self.style.muted_text
        };
        self.draw_text(
            format!("{prefix}{text}"),
            rect.position + ivec2(self.style.padding as i32, 2),
            color,
        );
    }
}
