use std::fmt::Debug;

use api::{BTN_L, BTN_R, Button as EngineButton};
use glam::ivec2;
use strum::IntoEnumIterator;

use super::Widget;
use crate::{Rect, Response, Ui};

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
        if ui.input().just_pressed(EngineButton::L) {
            selected_index = if selected_index == 0 {
                tabs.len() - 1
            } else {
                selected_index - 1
            };
        } else if ui.input().just_pressed(EngineButton::R) {
            selected_index = (selected_index + 1) % tabs.len();
        }
        *selected = tabs[selected_index];
        let changed = old != *selected;
        if changed {
            ui.reset_focus();
        }

        let rect = ui.row_rect();
        let side_width = if tabs.len() > 1 {
            ui.style.row_height.min(rect.size.x / 2)
        } else {
            0
        };
        let gap = 2u32;
        let labels: Vec<String> = tabs.iter().map(|tab| format!("{tab:?}")).collect();
        let tab_widths: Vec<u32> = labels
            .iter()
            .map(|label| {
                ui.text_width(label)
                    .saturating_add(ui.style.padding.saturating_mul(2))
            })
            .collect();
        let tabs_width = tab_widths
            .iter()
            .copied()
            .fold(0u32, u32::saturating_add)
            .saturating_add(gap.saturating_mul(tabs.len().saturating_sub(1) as u32));
        let bar_width = tabs_width.saturating_add(side_width.saturating_mul(2));
        let bar_x = rect.position.x + (rect.size.x as i32 - bar_width as i32) / 2;
        let mut x = bar_x + side_width as i32;

        for (index, (label, width)) in labels.iter().zip(tab_widths).enumerate() {
            let active = index == selected_index;
            let tab_rect = Rect::new(x, rect.position.y, width, rect.size.y);
            ui.draw_rect(
                tab_rect,
                if active {
                    ui.style.widget_focused
                } else {
                    ui.style.widget
                },
                ui.style.corner_radius,
            );
            ui.draw_centered_text(
                tab_rect,
                label,
                if active {
                    ui.style.focused_text
                } else {
                    ui.style.text
                },
            );
            x += width as i32
                + if index + 1 < tabs.len() {
                    gap as i32
                } else {
                    0
                };
        }

        if tabs.len() > 1 {
            ui.draw_text(BTN_L, ivec2(bar_x + 2, rect.position.y + 2), ui.style.text);
            ui.draw_text(
                BTN_R,
                ivec2(
                    bar_x + bar_width as i32 - side_width as i32 + 4,
                    rect.position.y + 2,
                ),
                ui.style.text,
            );
        }

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
