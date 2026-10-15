use std::fmt::Debug;

use api::{Button as EngineButton, strum::IntoEnumIterator};

use super::Widget;
use crate::{Response, Ui};

pub struct ComboBox<'a, T> {
    label: String,
    selected: &'a mut T,
}

impl<'a, T> ComboBox<'a, T> {
    pub fn new(label: impl Into<String>, selected: &'a mut T) -> Self {
        Self {
            label: label.into(),
            selected,
        }
    }
}

impl<T> Widget for ComboBox<'_, T>
where
    T: IntoEnumIterator + Copy + PartialEq + Debug,
{
    fn show(self, ui: &mut Ui) -> Response {
        let options: Vec<T> = T::iter().collect();
        let mut response = ui.widget_response();
        let rect = ui.row_rect();
        if options.is_empty() {
            ui.draw_interactive(rect, &format!("({})", self.label), false, response);
            ui.advance_row();
            return response;
        }

        let old = *self.selected;
        let current_index = options
            .iter()
            .position(|option| *option == old)
            .unwrap_or(0);
        let next_index = if response.focused && ui.input().just_pressed(EngineButton::A) {
            (current_index + 1) % options.len()
        } else {
            current_index
        };
        *self.selected = options[next_index];
        let changed = old != *self.selected;

        let text = format!("{}: {:?}", self.label, *self.selected);
        ui.draw_interactive(rect, &text, false, response);
        ui.advance_row();
        response.changed = changed;
        response
    }
}

impl Ui {
    pub fn combo_box<T>(&mut self, label: impl Into<String>, selected: &mut T) -> Response
    where
        T: IntoEnumIterator + Copy + PartialEq + Debug,
    {
        self.add(ComboBox::new(label, selected))
    }
}
