mod metadata;
mod update;

use std::sync::mpsc::{self, Receiver, TryRecvError};

use api::{
    Button, Color, Frame, Screen, ScreenAction, ScreenContext,
    glam::{ivec2, uvec2},
};
use ui::{Alignment, Ui, VerticalAlignment};

#[derive(Default)]
struct State;

api::impl_app_state!(State);

struct UpdaterScreen {
    ui: Ui,
    updates: Option<Receiver<update::Progress>>,
    status: (String, Color),
    current_step: Option<update::UpdateStep>,
    error: Option<String>,
    started: bool,
    finished: bool,
}

impl UpdaterScreen {
    fn new() -> Self {
        let mut ui = Ui::new();
        ui.set_alignment(Alignment::Center);
        ui.set_vertical_alignment(VerticalAlignment::Center);
        let text_color = ui.style.text;

        Self {
            ui,
            updates: None,
            status: ("Ready".into(), text_color),
            current_step: None,
            error: None,
            started: false,
            finished: false,
        }
    }

    fn poll_updates(&mut self) {
        let Some(updates) = &self.updates else {
            return;
        };
        loop {
            match updates.try_recv() {
                Ok(update::Progress::Step(step)) => {
                    self.status.0 = step.title().to_owned();
                    self.current_step = Some(step);
                }
                Ok(update::Progress::Finished) => {
                    self.status = ("Update complete".into(), self.ui.style.success);
                    self.finished = true;
                }
                Ok(update::Progress::Failed(error)) => {
                    self.status = ("Update failed".into(), self.ui.style.error);
                    self.error = Some(error);
                    self.finished = true;
                }
                Err(TryRecvError::Empty) => return,
                Err(TryRecvError::Disconnected) => {
                    if !self.finished {
                        self.status = ("Update stopped unexpectedly".into(), self.ui.style.warn);
                        self.error = Some("The update stopped before it reported a result.".into());
                        self.finished = true;
                    }
                    return;
                }
            }
        }
    }

    fn start_update(&mut self) {
        let (sender, updates) = mpsc::channel();
        self.updates = Some(updates);
        self.started = true;
        self.status.0 = "Starting update".into();
        std::thread::spawn(move || {
            let result = update::run(|step| {
                let _ = sender.send(update::Progress::Step(step));
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
        self.poll_updates();
        self.ui.begin_frame(ctx);

        if let Some(step) = &self.current_step {
            if self.finished {
                self.ui.colored_label(&self.status.0, self.status.1);
            } else {
                self.ui.colored_label(step.title(), self.status.1);
            }
            for line in wrap_text(&step.detail(), 34) {
                self.ui.small(line);
            }
        } else {
            self.ui.colored_label(&self.status.0, self.status.1);
        }
        if let Some(error) = &self.error {
            for line in wrap_text(error, 34) {
                self.ui.label(line);
            }
        }

        if !self.started {
            if self.ui.button("Start update").clicked {
                self.start_update();
            }
            if self.ui.button("Exit").clicked || ctx.input.just_pressed(Button::B) {
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
        frame.rectangle(
            ivec2(0, 0),
            uvec2(frame.width(), frame.height()),
            Color::Black,
        );
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
