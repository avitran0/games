use api::{HEIGHT, WIDTH};
use ui::{Alignment, Rect, Ui};

use crate::{main_menu::MainMenu, state::State};

pub struct PauseScreen {
    ui: Ui,
}

impl PauseScreen {
    pub fn new() -> Self {
        let mut ui = Ui::new();
        ui.set_alignment(Alignment::Center);
        ui.set_bounds(Rect::new(
            WIDTH as i32 / 4,
            HEIGHT as i32 / 4,
            WIDTH / 2,
            HEIGHT / 2,
        ));

        Self { ui }
    }
}

impl api::Screen<State> for PauseScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        self.ui.begin_frame(ctx);

        self.ui.label("Paused");

        if self.ui.button("Continue").clicked {
            return api::ScreenAction::Pop;
        }

        if self.ui.button("Quit").clicked {
            let menu = match MainMenu::new(ctx) {
                Ok(menu) => menu,
                Err(action) => return action,
            };
            return api::ScreenAction::ClearAndPush(Box::new(menu));
        }

        api::ScreenAction::None
    }

    fn draw(&mut self, _state: &State, frame: &mut api::Frame) {
        self.ui.paint(frame);
    }

    fn is_modal(&self) -> bool {
        true
    }
}
