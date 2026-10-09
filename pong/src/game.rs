use api::{Anchor, Button, Color, HEIGHT, WIDTH, glam::ivec2};
use ui::{EnumIter, Label, ProgressBar, Rect, Ui};

use crate::state::State;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter)]
#[strum(crate = "ui")]
enum DemoTab {
    Widgets,
    Layout,
}

pub struct GameScreen {
    ui: Ui,
    tab: DemoTab,
    sound: bool,
    volume: f32,
    lives: u8,
    speed: u8,
    clicks: u32,
    scroll_offset: u32,
    pressed_buttons: String,
}

impl GameScreen {
    pub fn new(_ctx: &mut api::ScreenContext<State>) -> Self {
        Self {
            ui: Ui::new(),
            tab: DemoTab::Widgets,
            sound: true,
            volume: 0.65,
            lives: 3,
            speed: 1,
            clicks: 0,
            scroll_offset: 0,
            pressed_buttons: String::new(),
        }
    }
}

impl api::Screen<State> for GameScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        if ctx.input.is_pressed(Button::Start) && ctx.input.is_pressed(Button::Select) {
            return api::ScreenAction::Quit;
        }

        self.ui.begin_frame(ctx.input);
        self.pressed_buttons = pressed_button_icons(ctx.input);

        let Self {
            ui,
            tab,
            sound,
            volume,
            lives,
            speed,
            clicks,
            scroll_offset,
            ..
        } = self;

        ui.background(Rect::new(0, 0, WIDTH, HEIGHT), Color::CodGray);
        ui.tab_bar(tab, |ui, active_tab| match active_tab {
            DemoTab::Widgets => {
                ui.add(Label::new("BUTTONS AND VALUES").heading());
                ui.horizontal(2, 4, |ui| {
                    if ui.button(format!("Button ({clicks})")).clicked {
                        *clicks = clicks.saturating_add(1);
                    }
                    ui.checkbox("Sound", sound);
                });
                ui.horizontal(2, 4, |ui| {
                    ui.slider("Volume", volume, 0.0_f32, 1.0_f32);
                    ui.slider("Lives", lives, 1_u8, 9_u8);
                });
                ui.horizontal(3, 3, |ui| {
                    ui.radio("Slow", speed, 0_u8);
                    ui.radio("Normal", speed, 1_u8);
                    ui.radio("Fast", speed, 2_u8);
                });
                ui.add(ProgressBar::new(*volume).label("Volume level"));
            }
            DemoTab::Layout => {
                ui.add(Label::new("SCROLL WITH BUTTON FOCUS").heading());
                ui.scroll_area(Rect::new(4, 41, WIDTH - 8, 64), scroll_offset, |ui| {
                    for item in 0..8 {
                        ui.button(format!("Scrollable row {}", item + 1));
                    }
                });
                ui.panel(Rect::new(4, 120, WIDTH - 8, 36), "Nested panel", |ui| {
                    ui.label("No border, no outside margin.");
                });
            }
        });

        api::ScreenAction::None
    }

    fn draw(&mut self, _state: &State, frame: &mut api::Frame) {
        self.ui.paint(frame);
        let display = if self.pressed_buttons.is_empty() {
            "INPUT: -".to_owned()
        } else {
            format!("INPUT: {}", self.pressed_buttons)
        };
        frame.text(
            display,
            ivec2(WIDTH as i32 - 4, HEIGHT as i32 - 4),
            Anchor::BottomRight,
        );
    }
}

fn pressed_button_icons(input: &api::Input) -> String {
    [
        (Button::Up, api::BTN_UP),
        (Button::Down, api::BTN_DOWN),
        (Button::Left, api::BTN_LEFT),
        (Button::Right, api::BTN_RIGHT),
        (Button::A, api::BTN_A),
        (Button::B, api::BTN_B),
        (Button::X, api::BTN_X),
        (Button::Y, api::BTN_Y),
        (Button::L, api::BTN_L),
        (Button::R, api::BTN_R),
        (Button::Start, api::BTN_START),
        (Button::Select, api::BTN_SELECT),
    ]
    .into_iter()
    .filter_map(|(button, icon)| input.is_pressed(button).then_some(icon))
    .collect()
}
