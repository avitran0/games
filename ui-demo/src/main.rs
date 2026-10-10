use api::{
    Color, Frame, HEIGHT, Screen, ScreenAction, ScreenContext, WIDTH,
    glam::{ivec2, uvec2},
};
use ui::{EnumIter, Rect, Ui};

#[derive(Default)]
struct State {
    page: Page,
    click_count: u32,
    enabled: bool,
    choice: Choice,
    volume: u8,
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

struct Demo {
    ui: Ui,
}

impl Demo {
    fn new() -> Self {
        Self { ui: Ui::new() }
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
                ui.checkbox("Enable option", enabled);
                ui.radio("First choice", choice, Choice::First);
                ui.radio("Second choice", choice, Choice::Second);
                ui.radio("Third choice", choice, Choice::Third);
                ui.slider("Volume", volume, 0, 10);
            }
            Page::Layout => {
                ui.heading("Layout and scrolling");
                ui.horizontal(2, 4, |ui| {
                    ui.button("Left");
                    ui.button("Right");
                });
                ui.columns(2, 4, |ui, column| {
                    ui.label(format!("Column {}", column + 1));
                    ui.button(if column == 0 { "One" } else { "Two" });
                });
            }
        });

        ScreenAction::None
    }

    fn draw(&mut self, _state: &State, frame: &mut Frame) {
        frame.rectangle(ivec2(0, 0), uvec2(WIDTH, HEIGHT), Color::Black);
        self.ui.paint(frame);
    }
}

fn main() {
    api::run("UI demo", State::default(), |_ctx| {
        Ok(Box::new(Demo::new()))
    });
}
