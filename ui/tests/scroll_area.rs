use ui::{Rect, Ui};

#[test]
fn scroll_area_clamps_offset_to_content() {
    let mut ui = Ui::new();
    ui.begin_frame(&api::Input::default());
    let mut offset = 1000;
    ui.scroll_area(Rect::new(0, 0, 80, 20), &mut offset, |ui| {
        ui.label("One");
        ui.label("Two");
        ui.label("Three");
    });
    assert_eq!(offset, 22);
}
