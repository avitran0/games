use ui::{Rect, Ui};

#[test]
fn remaining_space_tracks_the_layout_cursor() {
    let mut ui = Ui::new();
    ui.set_bounds(Rect::new(10, 20, 100, 80));
    ui.add_space(15);

    assert_eq!(ui.remaining(), Rect::new(10, 35, 100, 65));
}

#[test]
fn remaining_space_is_empty_after_passing_the_bounds() {
    let mut ui = Ui::new();
    ui.set_bounds(Rect::new(10, 20, 100, 30));
    ui.add_space(40);

    assert_eq!(ui.remaining(), Rect::new(10, 60, 100, 0));
}
