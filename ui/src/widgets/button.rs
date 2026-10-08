use super::Widget;
use crate::{Response, Ui};

pub struct Button {
    label: String,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }
}

impl Widget for Button {
    fn show(self, ui: &mut Ui) -> Response {
        let response = ui.widget_response();
        let rect = ui.row_rect();
        ui.draw_interactive(rect, &self.label, false, response);
        ui.advance_row();
        response
    }
}

impl Ui {
    pub fn button(&mut self, label: impl Into<String>) -> Response {
        self.add(Button::new(label))
    }
}
