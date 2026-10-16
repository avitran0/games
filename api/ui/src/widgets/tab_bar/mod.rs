use std::fmt::Debug;

use api::strum::IntoEnumIterator;

use super::Widget;
use crate::{Response, Ui};

mod layout;

pub struct TabBar<'a, T, F> {
    selected: &'a mut T,
    contents: F,
}

impl<'a, T, F> TabBar<'a, T, F>
where
    F: FnOnce(&mut Ui, T),
{
    pub fn new(selected: &'a mut T, contents: F) -> Self {
        Self { selected, contents }
    }
}

impl<T, F> Widget for TabBar<'_, T, F>
where
    T: IntoEnumIterator + Copy + PartialEq + Debug,
    F: FnOnce(&mut Ui, T),
{
    fn show(self, ui: &mut Ui) -> Response {
        let Self { selected, contents } = self;
        let tabs: Vec<T> = T::iter().collect();
        let old = *selected;
        if tabs.is_empty() {
            ui.advance_row();
            contents(ui, old);
            return Response::default();
        }

        let mut selected_index = tabs.iter().position(|tab| *tab == old).unwrap_or(0);
        if ui.input().just_pressed(api::Button::L) {
            selected_index = if selected_index == 0 {
                tabs.len() - 1
            } else {
                selected_index - 1
            };
        } else if ui.input().just_pressed(api::Button::R) {
            selected_index = (selected_index + 1) % tabs.len();
        }
        *selected = tabs[selected_index];
        let changed = old != *selected;
        if changed {
            ui.reset_focus();
        }

        layout::draw_tabs(ui, &tabs, selected_index);
        ui.advance_row();
        let response = Response {
            changed,
            ..Response::default()
        };
        contents(ui, *selected);
        response
    }
}

impl Ui {
    pub fn tab_bar<T>(&mut self, selected: &mut T, contents: impl FnOnce(&mut Ui, T)) -> Response
    where
        T: IntoEnumIterator + Copy + PartialEq + Debug,
    {
        self.add(TabBar::new(selected, contents))
    }
}
