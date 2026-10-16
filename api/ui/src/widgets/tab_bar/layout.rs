use std::fmt::Debug;

use api::{BTN_L, BTN_R, glam::ivec2};

use crate::{Rect, Ui};

pub(super) fn draw_tabs<T: Debug>(ui: &mut Ui, tabs: &[T], selected_index: usize) {
    let row = ui.row_rect();
    let arrow_width = if tabs.len() > 1 {
        ui.style.row_height.min(row.size.x / 2)
    } else {
        0
    };
    let viewport = Rect::new(
        row.position.x + arrow_width as i32,
        row.position.y,
        row.size.x.saturating_sub(arrow_width.saturating_mul(2)),
        row.size.y,
    );
    let gap = 2_i32;
    let center_label = format!("{:?}", tabs[selected_index]);
    let center_size = fit_tab(ui, &center_label, viewport.size.x);
    let center = center_size.map(|(label, width)| {
        let rect = Rect::new(
            centered_x(viewport.position.x, viewport.size.x, width),
            viewport.position.y,
            width,
            viewport.size.y,
        );
        draw_tab(ui, rect, &label, true);
        rect
    });

    let (left_index, right_index) = neighbor_indices(tabs.len(), selected_index);
    if let Some(center) = center {
        let left_space = (center.position.x - viewport.position.x - gap).max(0) as u32;
        let right_space = (viewport.position.x + viewport.size.x as i32
            - center.position.x
            - center.size.x as i32
            - gap)
            .max(0) as u32;
        let side_width = left_space.min(right_space);
        let left_area = Rect::new(
            center.position.x - gap - side_width as i32,
            viewport.position.y,
            side_width,
            viewport.size.y,
        );
        let right_area = Rect::new(
            center.position.x + center.size.x as i32 + gap,
            viewport.position.y,
            side_width,
            viewport.size.y,
        );

        if let Some(index) = left_index {
            draw_neighbor(ui, tabs, index, left_area, side_width);
        }
        if let Some(index) = right_index {
            draw_neighbor(ui, tabs, index, right_area, side_width);
        }
    }

    if tabs.len() > 1 {
        ui.draw_text(
            BTN_L,
            ivec2(row.position.x + 2, row.position.y + 2),
            ui.style.text,
        );
        ui.draw_text(
            BTN_R,
            ivec2(
                row.position.x + row.size.x as i32 - arrow_width as i32 + 4,
                row.position.y + 2,
            ),
            ui.style.text,
        );
    }
}

fn draw_neighbor<T: Debug>(ui: &mut Ui, tabs: &[T], index: usize, area: Rect, max_width: u32) {
    let label = format!("{:?}", tabs[index]);
    if let Some((label, width)) = fit_tab(ui, &label, max_width) {
        let rect = Rect::new(
            centered_x(area.position.x, area.size.x, width),
            area.position.y,
            width,
            area.size.y,
        );
        draw_tab(ui, rect, &label, false);
    }
}

fn centered_x(start: i32, area_width: u32, item_width: u32) -> i32 {
    start + (area_width.saturating_sub(item_width) / 2) as i32
}

fn draw_tab(ui: &mut Ui, rect: Rect, label: &str, active: bool) {
    ui.draw_rect(
        rect,
        if active {
            ui.style.widget_focused
        } else {
            ui.style.widget
        },
        ui.style.corner_radius,
    );
    ui.draw_centered_text(
        rect,
        label,
        if active {
            ui.style.focused_text
        } else {
            ui.style.widget_text
        },
    );
}

fn fit_tab(ui: &Ui, label: &str, max_width: u32) -> Option<(String, u32)> {
    let text_width = max_width.saturating_sub(ui.style.padding.saturating_mul(2));
    if max_width < ui.style.padding.saturating_mul(2) {
        return None;
    }
    let label = fit_label(ui, label, text_width);
    let width = pill_width(ui.text_width(&label), ui.style.padding, max_width)?;
    Some((label, width))
}

fn pill_width(text_width: u32, padding: u32, max_width: u32) -> Option<u32> {
    let padded_width = text_width.saturating_add(padding.saturating_mul(2));
    (max_width >= padding.saturating_mul(2)).then_some(padded_width.min(max_width))
}

fn fit_label(ui: &Ui, label: &str, max_width: u32) -> String {
    if ui.text_width(label) <= max_width {
        return label.to_owned();
    }

    let suffix = "...";
    let suffix_width = ui.text_width(suffix);
    if max_width < suffix_width {
        return String::new();
    }
    let prefix_width = max_width - suffix_width;
    let mut output = String::new();
    for character in label.chars() {
        let mut candidate = output.clone();
        candidate.push(character);
        if ui.text_width(&candidate) > prefix_width {
            break;
        }
        output.push(character);
    }
    output.push_str(suffix);
    output
}

fn neighbor_indices(count: usize, selected: usize) -> (Option<usize>, Option<usize>) {
    match count {
        0 | 1 => (None, None),
        2 => (None, Some((selected + 1) % count)),
        _ => (
            Some((selected + count - 1) % count),
            Some((selected + 1) % count),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{centered_x, neighbor_indices, pill_width};

    #[test]
    fn carousel_neighbors_wrap_at_both_ends() {
        assert_eq!(neighbor_indices(4, 0), (Some(3), Some(1)));
        assert_eq!(neighbor_indices(4, 3), (Some(2), Some(0)));
    }

    #[test]
    fn pills_are_centered_in_their_layout_slots() {
        assert_eq!(centered_x(10, 60, 20), 30);
        assert_eq!(centered_x(100, 40, 40), 100);
    }

    #[test]
    fn active_pill_uses_its_measured_text_width() {
        assert_eq!(pill_width(42, 6, 100), Some(54));
        assert_eq!(pill_width(120, 6, 100), Some(100));
        assert_eq!(pill_width(1, 6, 10), None);
    }

    #[test]
    fn carousel_does_not_duplicate_the_neighbor_for_two_tabs() {
        assert_eq!(neighbor_indices(2, 0), (None, Some(1)));
        assert_eq!(neighbor_indices(2, 1), (None, Some(0)));
        assert_eq!(neighbor_indices(1, 0), (None, None));
    }
}
