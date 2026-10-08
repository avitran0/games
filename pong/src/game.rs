use ui::Ui;

use crate::state::State;

pub struct GameScreen {
    ui: Ui,
    btn: bool,
}

impl GameScreen {
    const PADDLE_HEIGHT: u32 = 24;

    pub fn new(ctx: &mut api::ScreenContext<State>) -> Self {
        Self {
            ui: Ui::new(),
            btn: false,
        }
    }
}

impl api::Screen<State> for GameScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        self.ui.begin_frame(ctx.input);

        self.ui.add(ui::Button::new("Button"));
        self.ui.add(ui::Checkbox::new("Chk", &mut self.btn));
        self.ui.add(ui::ProgressBar::new(0.5).label("Progress"));

        api::ScreenAction::None
    }

    fn draw(&mut self, state: &State, frame: &mut api::Frame) {
        self.ui.paint(frame);
    }
}
