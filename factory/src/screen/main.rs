use ui::Ui;

use crate::state::State;

pub struct MainMenu {
    ui: Ui,
}

impl MainMenu {
    pub fn new() -> Self {
        Self { ui: Ui::new() }
    }
}

impl api::Screen<State> for MainMenu {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        self.ui.begin_frame(ctx.input);

        self.ui.button("Play");

        if self.ui.button("Quit").clicked {
            return api::ScreenAction::Quit;
        }

        api::ScreenAction::None
    }

    fn draw(&mut self, state: &State, frame: &mut api::Frame) {
        self.ui.paint(frame);
    }
}
