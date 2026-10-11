use api::{HEIGHT, WIDTH};
use ui::{Alignment, Rect, Ui};

use crate::state::State;

pub struct MainMenu {
    ui: Ui,
}

impl MainMenu {
    pub fn new() -> Self {
        let mut ui = Ui::new();
        ui.set_alignment(Alignment::Center);
        Self { ui }
    }
}

impl api::Screen<State> for MainMenu {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        self.ui.begin_frame(ctx);
        self.ui.set_bounds(Rect::new(0, 60, WIDTH, HEIGHT - 60));

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
