use api::{RADIO_CHECKED, RADIO_UNCHECKED};

use super::Widget;
use crate::{Response, Ui};

pub struct Radio<'a, T> {
    label: String,
    value: &'a mut T,
    choice: T,
}

impl<'a, T> Radio<'a, T> {
    pub fn new(label: impl Into<String>, value: &'a mut T, choice: T) -> Self {
        Self {
            label: label.into(),
            value,
            choice,
        }
    }
}

impl<T: PartialEq + Copy> Widget for Radio<'_, T> {
    fn show(self, ui: &mut Ui) -> Response {
        let selected = *self.value == self.choice;
        let response = ui.widget_response();
        if response.clicked {
            *self.value = self.choice;
        }
        let label = format!(
            "{} {}",
            if *self.value == self.choice {
                RADIO_CHECKED
            } else {
                RADIO_UNCHECKED
            },
            self.label
        );
        let rect = ui.row_rect();
        ui.draw_interactive(
            rect,
            &label,
            selected || *self.value == self.choice,
            response,
        );
        ui.advance_row();
        Response {
            changed: response.clicked && !selected,
            ..response
        }
    }
}

impl Ui {
    pub fn radio<T: PartialEq + Copy>(
        &mut self,
        label: impl Into<String>,
        value: &mut T,
        choice: T,
    ) -> Response {
        self.add(Radio::new(label, value, choice))
    }
}
