use api::{HEIGHT, WIDTH};
use ui::{Alignment, Rect, Ui};

use crate::{screen::game::GameScreen, state::State};

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

        if self.ui.button("Play").clicked {
            return match GameScreen::new(ctx) {
                Ok(screen) => api::ScreenAction::Replace(Box::new(screen)),
                Err(action) => action,
            };
        }

        if self.ui.button("Quit").clicked {
            return api::ScreenAction::Quit;
        }

        api::ScreenAction::None
    }

    fn draw(&mut self, _state: &State, frame: &mut api::Frame) {
        self.ui.paint(frame);
    }
}
