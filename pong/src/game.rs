use api::{Anchor, Button, HEIGHT, WIDTH, glam::ivec2};
use ui::Ui;

use crate::state::State;

pub struct GameScreen {
    ui: Ui,
    btn: bool,
    pressed_buttons: String,
}

impl GameScreen {
    const PADDLE_HEIGHT: u32 = 24;

    pub fn new(_ctx: &mut api::ScreenContext<State>) -> Self {
        Self {
            ui: Ui::new(),
            btn: false,
            pressed_buttons: String::new(),
        }
    }
}

impl api::Screen<State> for GameScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        self.ui.begin_frame(ctx.input);
        self.pressed_buttons = pressed_button_icons(ctx.input);

        self.ui.add(ui::Button::new("Button"));
        self.ui.add(ui::Checkbox::new("Chk", &mut self.btn));
        self.ui.add(ui::ProgressBar::new(0.5).label("Progress"));

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
            ivec2(WIDTH as i32 - 5, HEIGHT as i32 - 5),
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
