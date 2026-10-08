use std::fmt::Debug;

use api::Button as EngineButton;
use glam::ivec2;
use strum::IntoEnumIterator;

use super::Widget;
use crate::{Response, Ui};

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
        let mut response = ui.widget_response();
        let tabs: Vec<T> = T::iter().collect();
        let old = *selected;
        if tabs.is_empty() {
            ui.advance_row();
            contents(ui, old);
            return response;
        }

        let mut selected_index = tabs.iter().position(|tab| *tab == old).unwrap_or(0);
        if response.focused {
            if ui.input().just_pressed(EngineButton::L) {
                selected_index = if selected_index == 0 {
                    tabs.len() - 1
                } else {
                    selected_index - 1
                };
            } else if ui.input().just_pressed(EngineButton::R) {
                selected_index = (selected_index + 1) % tabs.len();
            }
        }
        *selected = tabs[selected_index];

        let rect = ui.row_rect();
        let count = tabs.len() as u32;
        let base_width = rect.size.x / count;
        let remainder = rect.size.x % count;
        let mut x = rect.position.x;
        for (index, tab) in tabs.iter().enumerate() {
            let width = base_width + u32::from(index + 1 == tabs.len()) * remainder;
            let active = index == selected_index;
            let fill = if active {
                ui.style.widget_focused
            } else {
                ui.style.widget
            };
            let border = if active && response.focused {
                ui.style.accent
            } else {
                ui.style.panel_border
            };
            let tab_rect = crate::Rect::new(x, rect.position.y, width, rect.size.y);
            ui.draw_bordered_rect(
                tab_rect,
                fill,
                border,
                ui.style.border_width,
                ui.style.corner_radius,
            );
            let prefix = if active && response.focused { "> " } else { "" };
            let color = if active {
                ui.style.text
            } else {
                ui.style.muted_text
            };
            ui.draw_text(
                format!("{prefix}{tab:?}"),
                tab_rect.position + ivec2(ui.style.padding as i32, 1),
                color,
            );
            x += width as i32;
        }
        ui.advance_row();
        response.changed = old != *selected;
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
