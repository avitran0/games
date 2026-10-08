use super::Widget;
use crate::{Response, Ui};

pub struct Selectable {
    label: String,
    selected: bool,
}

impl Selectable {
    pub fn new(label: impl Into<String>, selected: bool) -> Self {
        Self {
            label: label.into(),
            selected,
        }
    }
}

impl Widget for Selectable {
    fn show(self, ui: &mut Ui) -> Response {
        let response = ui.widget_response();
        let rect = ui.row_rect();
        ui.draw_interactive(rect, &self.label, self.selected, response);
        ui.advance_row();
        response
    }
}

impl Ui {
    pub fn selectable(&mut self, label: impl Into<String>, selected: bool) -> Response {
        self.add(Selectable::new(label, selected))
    }
}
