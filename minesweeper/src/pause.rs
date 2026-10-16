use api::{
    Color, HEIGHT, WIDTH,
    glam::{ivec2, uvec2},
};
use ui::{Alignment, Rect, Ui};

use crate::{menu::MainMenu, state::State};

pub struct PauseScreen {
    ui: Ui,
    confirming_quit: bool,
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

        Self {
            ui,
            confirming_quit: false,
        }
    }
}

impl api::Screen<State> for PauseScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        self.ui.begin_frame(ctx);

        if self.confirming_quit {
            self.confirm(ctx)
        } else {
            self.menu()
        }
    }

    fn draw(&mut self, _state: &State, frame: &mut api::Frame) {
        frame.rectangle_with_opacity(ivec2(0, 0), uvec2(WIDTH, HEIGHT), Color::Black, 0.1);
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
