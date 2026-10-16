mod metadata;
mod update;

use std::sync::mpsc::{self, Receiver};

use api::{
    Button, Color, Frame, Screen, ScreenAction, ScreenContext,
    glam::{ivec2, uvec2},
};
use ui::{Alignment, Rect, Ui, VerticalAlignment};

#[derive(Default)]
struct State;

api::impl_app_state!(State);

struct UpdaterScreen {
    ui: Ui,
    updates: Option<Receiver<update::Progress>>,
    status: String,
    error: Option<String>,
    started: bool,
    finished: bool,
}

impl UpdaterScreen {
    fn new() -> Self {
        let mut ui = Ui::new();
        ui.set_alignment(Alignment::Center);
        ui.set_vertical_alignment(VerticalAlignment::Center);
        ui.set_bounds(Rect::new(0, 8, api::WIDTH, api::HEIGHT - 16));
        Self {
            ui,
            updates: None,
            status: "Ready".into(),
            error: None,
            started: false,
            finished: false,
        }
    }

    fn start_update(&mut self) {
        let (sender, updates) = mpsc::channel();
        self.updates = Some(updates);
        self.started = true;
        self.status = "Starting update".into();
        std::thread::spawn(move || {
            let result = update::run(|message| {
                let _ = sender.send(update::Progress::Status(message.to_owned()));
            });
            let _ = sender.send(match result {
                Ok(()) => update::Progress::Finished,
                Err(error) => update::Progress::Failed(error),
            });
        });
    }
}

impl Screen<State> for UpdaterScreen {
    fn update(&mut self, ctx: &mut ScreenContext<'_, State>) -> ScreenAction<State> {
        if let Some(updates) = &self.updates {
            while let Ok(progress) = updates.try_recv() {
                match progress {
                    update::Progress::Status(message) => self.status = message,
                    update::Progress::Finished => {
                        self.status = "Update complete".into();
                        self.finished = true;
                    }
                    update::Progress::Failed(error) => {
                        self.status = "Update failed".into();
                        self.error = Some(error);
                        self.finished = true;
                    }
                }
            }
        }

        self.ui.begin_frame(ctx);
        self.ui.heading("Updater");
        self.ui.label(&self.status);
        if let Some(error) = &self.error {
            for line in wrap_text(error, 34) {
                self.ui.label(line);
            }
        }

        if !self.started {
            if self.ui.button("Start update").clicked {
                self.start_update();
            }
            if ctx.input.just_pressed(Button::B) {
                return ScreenAction::Quit;
            }
        } else if self.finished
            && (self.ui.button("Close").clicked || ctx.input.just_pressed(Button::B))
        {
            return ScreenAction::Quit;
        }
        ScreenAction::None
    }

    fn draw(&mut self, _state: &State, frame: &mut Frame) {
        frame.rectangle(ivec2(0, 0), uvec2(api::WIDTH, api::HEIGHT), Color::Black);
        self.ui.paint(frame);
    }
}

fn wrap_text(text: &str, limit: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let extra = usize::from(!line.is_empty()) + word.chars().count();
        if !line.is_empty() && line.chars().count() + extra > limit {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    if lines.is_empty() {
        lines.push(text.to_owned());
    }
    lines
}

fn main() {
    api::run("Updater", State, |_ctx| Ok(Box::new(UpdaterScreen::new())));
}
