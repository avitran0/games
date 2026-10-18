use api::{Color, glam::ivec2};
use ui::{Alignment, Ui, VerticalAlignment};

use crate::{menu::MainMenu, state::State};

pub struct PauseScreen {
    ui: Ui,
    confirming_quit: bool,
}

impl PauseScreen {
    pub fn new() -> Self {
        let mut ui = Ui::new();
        ui.set_alignment(Alignment::Center);
        ui.set_vertical_alignment(VerticalAlignment::Center);

        Self {
            ui,
            confirming_quit: false,
        }
    }
}

impl api::Screen<State> for PauseScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        ctx.state.camera.set_resolution(ctx.resolution);
        self.ui.begin_frame(ctx);

        if self.confirming_quit {
            self.confirm(ctx)
        } else {
            self.menu()
        }
    }

    fn draw(&mut self, _state: &State, frame: &mut api::Frame) {
        frame.rectangle_with_opacity(ivec2(0, 0), frame.resolution().into(), Color::Black, 0.1);
        self.ui.paint(frame);
    }

    fn is_modal(&self) -> bool {
        true
    }
}

impl PauseScreen {
    fn menu(&mut self) -> api::ScreenAction<State> {
        self.ui.label("Paused");

        if self.ui.button("Continue").clicked {
            return api::ScreenAction::Pop;
        }

        if self.ui.button("Quit").clicked {
            self.confirming_quit = true;
        }

        api::ScreenAction::None
    }

    fn confirm(&mut self, ctx: &mut api::ScreenContext<State>) -> api::ScreenAction<State> {
        self.ui.label("Quit to main menu?");

        if self.ui.button("Quit").clicked {
            return api::ScreenAction::ClearAndPush(Box::new(MainMenu::new()));
        }

        if self.ui.button("Cancel").clicked || ctx.input.just_pressed(api::Button::B) {
            self.confirming_quit = false;
            self.ui.focus_first();
        }

        api::ScreenAction::None
    }
}
