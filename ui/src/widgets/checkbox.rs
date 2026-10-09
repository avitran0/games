use api::{CHECKBOX_CHECKED, CHECKBOX_UNCHECKED};

use super::Widget;
use crate::{Response, Ui};

pub struct Checkbox<'a> {
    label: String,
    value: &'a mut bool,
}

impl<'a> Checkbox<'a> {
    pub fn new(label: impl Into<String>, value: &'a mut bool) -> Self {
        Self {
            label: label.into(),
            value,
        }
    }
}

impl Widget for Checkbox<'_> {
    fn show(self, ui: &mut Ui) -> Response {
        let response = ui.widget_response();
        if response.clicked {
            *self.value = !*self.value;
        }
        let label = format!(
            "{} {}",
            if *self.value {
                CHECKBOX_CHECKED
            } else {
                CHECKBOX_UNCHECKED
            },
            self.label
        );
        let rect = ui.row_rect();
        ui.draw_interactive(rect, &label, *self.value, response);
        ui.advance_row();
        Response {
            changed: response.clicked,
            ..response
        }
    }
}

impl Ui {
    pub fn checkbox(&mut self, label: impl Into<String>, value: &mut bool) -> Response {
        self.add(Checkbox::new(label, value))
    }
}
