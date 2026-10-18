use ui::{Rect, Ui};

#[test]
fn scroll_area_clamps_offset_to_content() {
    let mut ui = Ui::new();
    let mut offset = 1000;
    ui.scroll_area(Rect::new(0, 0, 80, 20), &mut offset, |ui| {
        ui.label("One");
        ui.label("Two");
        ui.label("Three");
    });
    assert_eq!(offset, ui.style.row_height * 3 - 20);
}

#[test]
fn scroll_area_reserves_scrollbar_space_during_layout() {
    let mut ui = Ui::new();
    let mut offset = 0;
    ui.scroll_area(Rect::new(0, 0, 80, 100), &mut offset, |ui| {
        assert_eq!(ui.remaining().size.x, 75);
    });
}

#[test]
fn scroll_area_handles_narrow_viewports() {
    for width in 0..=4 {
        let mut ui = Ui::new();
        let mut offset = 0;
        ui.scroll_area(Rect::new(0, 0, width, 8), &mut offset, |ui| {
            ui.label("One");
            ui.label("Two");
            ui.label("Three");
        });
    }
}
