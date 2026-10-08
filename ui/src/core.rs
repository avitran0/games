use api::{Button as EngineButton, Color, Frame, HEIGHT, Input, WIDTH};
use glam::{IVec2, ivec2};

use crate::{
    Rect, Response, Style,
    layout::{Layout, LayoutSnapshot},
    paint::{PaintList, inset_rect},
    widgets::{Label, Separator, Widget},
};

pub struct Ui {
    pub style: Style,
    input: Input,
    layout: Layout,
    root_bounds: Rect,
    panels: Vec<LayoutSnapshot>,
    paint: PaintList,
    widget_count: usize,
    previous_widget_count: usize,
    focus: usize,
    activated: bool,
}

impl Default for Ui {
    fn default() -> Self {
        Self::new()
    }
}

impl Ui {
    pub fn new() -> Self {
        let bounds = Rect::new(8, 8, WIDTH.saturating_sub(16), HEIGHT.saturating_sub(16));
        Self {
            style: Style::default(),
            input: Input::default(),
            layout: Layout {
                bounds,
                cursor: bounds.position,
            },
            root_bounds: bounds,
            panels: Vec::new(),
            paint: PaintList::default(),
            widget_count: 0,
            previous_widget_count: 0,
            focus: 0,
            activated: false,
        }
    }

    pub fn begin_frame(&mut self, input: &Input) {
        self.input = *input;
        self.previous_widget_count = self.widget_count;
        self.widget_count = 0;
        self.paint.clear();
        self.panels.clear();
        self.layout = Layout {
            bounds: self.root_bounds,
            cursor: self.root_bounds.position,
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
        };
    }

    pub fn add<W: Widget>(&mut self, widget: W) -> Response {
        widget.show(self)
    }

    pub fn background(&mut self, rect: Rect, color: Color) {
        self.paint.rect(rect, color, 0);
    }

    pub fn begin_panel(&mut self, rect: Rect, title: impl AsRef<str>) {
        self.panels.push(LayoutSnapshot(self.layout));
        self.paint.bordered_rect(
            rect,
            self.style.panel,
            self.style.panel_border,
            self.style.border_width,
            self.style.corner_radius,
        );
        let bounds = inset_rect(rect, self.style.padding);
        self.layout = Layout {
            bounds,
            cursor: bounds.position,
        };
        if !title.as_ref().is_empty() {
            self.add(Label::new(title.as_ref()));
            self.add(Separator);
        }
    }

    pub fn end_panel(&mut self) {
        if let Some(LayoutSnapshot(layout)) = self.panels.pop() {
            self.layout = layout;
        }
    }

    pub fn add_space(&mut self, pixels: u32) {
        self.layout.cursor.y += pixels as i32;
    }

    pub fn paint(&self, frame: &mut Frame) {
        self.paint.draw(frame);
    }

    pub(crate) fn input(&self) -> &Input {
        &self.input
    }

    pub(crate) fn widget_response(&mut self) -> Response {
        let response = Response {
            focused: self.widget_count == self.focus,
            ..Response::default()
        };
        self.widget_count += 1;
        Response {
            clicked: response.focused && self.activated,
            ..response
        }
    }

    pub(crate) fn row_rect(&self) -> Rect {
        crate::paint::row_rect(
            self.layout.cursor,
            self.layout.bounds.size.x,
            self.style.row_height,
        )
    }

    pub(crate) fn advance_row(&mut self) {
        self.layout.cursor.y += (self.style.row_height + self.style.spacing) as i32;
    }

    pub(crate) fn advance(&mut self, pixels: u32) {
        self.layout.cursor.y += pixels as i32;
    }

    pub(crate) fn draw_text(&mut self, text: impl Into<String>, position: IVec2, color: Color) {
        self.paint.text(text, position, color);
    }

    pub(crate) fn draw_line(&mut self, start: IVec2, end: IVec2, color: Color) {
        self.paint.line(start, end, color);
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
        let fill = if response.focused {
            if self.activated {
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
            self.style.accent
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
            rect.position + ivec2(self.style.padding as i32, 1),
            color,
        );
    }
}
