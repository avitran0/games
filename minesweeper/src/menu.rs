use api::glam::uvec2;

use crate::{game::GameScreen, state::State};

pub struct MainMenu {
    ui: ui::Ui,
    width: u16,
    height: u16,
}

impl MainMenu {
    pub fn new() -> Self {
        let mut ui = ui::Ui::new();
        ui.set_alignment(ui::Alignment::Center);
        ui.set_bounds(ui::Rect::new(0, 48, api::WIDTH, api::HEIGHT - 48));

        Self {
            ui,
            width: 30,
            height: 20,
        }
    }
}

impl api::Screen<State> for MainMenu {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        self.ui.begin_frame(ctx);

        if self.ui.button("Play").clicked {
            let game = match GameScreen::new(ctx, uvec2(self.width.into(), self.height.into())) {
                Ok(game) => game,
                Err(action) => return action,
            };
            return api::ScreenAction::Replace(Box::new(game));
        }

        self.ui.slider("Width", &mut self.width, 1, u16::MAX);
        self.ui.slider("Height", &mut self.height, 1, u16::MAX);

        if self.ui.button("Quit").clicked {
            return api::ScreenAction::Quit;
        }

        api::ScreenAction::None
    }

    fn draw(&mut self, _state: &State, frame: &mut api::Frame) {
        self.ui.paint(frame);
    }
}
