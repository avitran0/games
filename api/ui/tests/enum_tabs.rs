use ui::{EnumIter, IntoEnumIterator, Label, SliderValue, TabBar, Ui};

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter)]
#[strum(crate = "ui")]
enum Page {
    Home,
    Settings,
}

#[test]
fn enum_iter_tabs_use_typed_values() {
    assert_eq!(
        Page::iter().collect::<Vec<_>>(),
        [Page::Home, Page::Settings]
    );

    let mut ui = Ui::new();
    let mut page = Page::Home;
    ui.add(TabBar::new(&mut page, |ui, tab| match tab {
        Page::Home => {
            ui.add(Label::new("Home"));
        }
        Page::Settings => {
            ui.add(Label::new("Settings"));
        }
    }));
    assert_eq!(page, Page::Home);
}

#[test]
fn sliders_support_multiple_number_types() {
    let mut ui = Ui::new();
    let mut volume = 0.5_f32;
    let mut lives = 3_u8;
    ui.slider("Volume", &mut volume, 0.0, 1.0);
    ui.slider("Lives", &mut lives, 0, 9);
    assert_eq!(volume, 0.5);
    assert_eq!(lives, 3);
}

#[test]
fn integer_slider_fill_keeps_full_integer_precision() {
    assert_eq!(i64::MIN.fill_width(i64::MIN, i64::MAX, 100), 0);
    assert_eq!(0_i64.fill_width(i64::MIN, i64::MAX, 100), 50);
    assert_eq!(u64::MAX.fill_width(0, u64::MAX, 100), 100);
    assert_eq!(usize::MAX.fill_width(0, usize::MAX, 100), 100);
}
