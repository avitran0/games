use api::Color;

use super::Widget;
use crate::{Response, Ui};

enum Tone {
    Normal,
    Muted,
    Heading,
}

pub struct Label {
    text: String,
    color: Option<Color>,
    tone: Tone,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: None,
            tone: Tone::Normal,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn muted(mut self) -> Self {
        self.tone = Tone::Muted;
        self
    }

    pub fn heading(mut self) -> Self {
        self.tone = Tone::Heading;
        self
    }
}

impl Widget for Label {
    fn show(self, ui: &mut Ui) -> Response {
        let color = self.color.unwrap_or(match self.tone {
            Tone::Normal => ui.style.text,
            Tone::Muted => ui.style.muted_text,
            Tone::Heading => ui.style.accent,
        });
        ui.draw_text(self.text, ui.row_rect().position, color);
        let extra = if matches!(self.tone, Tone::Heading) {
            ui.style.spacing
        } else {
            0
        };
        ui.advance(ui.style.row_height + extra);
        Response::default()
    }
}

impl Ui {
    pub fn label(&mut self, text: impl Into<String>) -> Response {
        self.add(Label::new(text))
    }

    pub fn small(&mut self, text: impl Into<String>) -> Response {
        self.add(Label::new(text).muted())
    }

    pub fn heading(&mut self, text: impl Into<String>) -> Response {
        self.add(Label::new(text).heading())
    }
}
