use api::{
    Frame, HEIGHT, Screen, ScreenAction, ScreenContext, WIDTH,
    glam::{ivec2, uvec2},
};
use ui::{EnumIter, Ui};

#[derive(Default)]
struct State {
    page: Page,
    click_count: u32,
    enabled: bool,
    choice: Choice,
    volume: u8,
    scaling: Scaling,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, EnumIter)]
#[strum(crate = "ui")]
enum Page {
    #[default]
    Overview,
    Controls,
    Layout,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Choice {
    #[default]
    First,
    Second,
    Third,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, EnumIter)]
#[strum(crate = "ui")]
enum Scaling {
    #[default]
    Aspect,
    Stretch,
    Integer,
}

struct Demo {
    ui: Ui,
    scroll: u32,
}

impl Demo {
    fn new() -> Self {
        Self {
            ui: Ui::new(),
            scroll: 0,
        }
    }
}

impl Screen<State> for Demo {
    fn update(&mut self, ctx: &mut ScreenContext<'_, State>) -> ScreenAction<State> {
        self.ui.begin_frame(ctx);
        let State {
            page,
            click_count,
            enabled,
            choice,
            volume,
            scaling,
        } = &mut *ctx.state;

        self.ui.tab_bar(page, |ui, page| match page {
            Page::Overview => {
                ui.heading("UI crate demo");
                ui.label("A small tour of the game UI toolkit.");
                ui.add_space(4);
                if ui.button("Click me").clicked {
                    *click_count = click_count.saturating_add(1);
                }
                ui.label(format!("Button activations: {click_count}"));
                ui.progress_bar(*click_count as f32 / 10.0, "Progress");
            }
            Page::Controls => {
                ui.heading("Controls");
                ui.combo_box("Screen Scaling", scaling);
                ui.checkbox("Enable option", enabled);
                ui.horizontal(3, 3, |ui| {
                    ui.radio("First", choice, Choice::First);
                    ui.radio("Second", choice, Choice::Second);
                    ui.radio("Third", choice, Choice::Third);
                });
                ui.slider("Volume", volume, 0, 10);
            }
            Page::Layout => {
                ui.heading("Layout and scrolling");
                ui.horizontal(2, 4, |ui| {
                    ui.button("Left");
                    ui.button("Right");
                });
                ui.label("Elements are arranged in a simple vertical flow.");
                ui.scroll_area(ui.remaining(), &mut self.scroll, |ui| {
                    ui.button("One");
                    ui.button("Two");
                    ui.button("Three");
                    ui.button("Four");
                    ui.button("Five");
                });
            }
        });

        ScreenAction::None
    }

    fn draw(&mut self, _state: &State, frame: &mut Frame) {
        frame.rectangle(ivec2(0, 0), uvec2(WIDTH, HEIGHT), self.ui.style.background);
        self.ui.paint(frame);
    }
}

fn main() {
    api::run("UI demo", State::default(), |_ctx| {
        Ok(Box::new(Demo::new()))
    });
}
