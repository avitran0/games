use crate::{
    document::AssetKind,
    editors::{FileState, apply_pixel_edit},
    ui::{self, canvas::PixelMode},
};
use api::formats::SpriteDocument;
use eframe::egui::Ui;

pub struct Screen {
    pub file: FileState,
    document: SpriteDocument,
    state: State,
}

impl Screen {
    pub fn new(document: SpriteDocument, file: FileState) -> Self {
        Self {
            file,
            document,
            state: State::default(),
        }
    }

    pub fn tools(&mut self, ui: &mut Ui) -> bool {
        tools(ui, &mut self.state)
    }

    pub fn canvas_controls(&mut self, ui: &mut Ui) {
        ui::canvas::controls(ui, &mut self.state.canvas);
    }

    pub fn canvas(&mut self, ui: &mut Ui) -> bool {
        canvas(ui, &mut self.document, &mut self.state)
    }

    pub fn info(&self) -> String {
        format!(
            "Static sprite: {}x{} px",
            self.document.size.x, self.document.size.y
        )
    }

    pub fn save(&mut self, save_as: bool) {
        self.file.save(
            AssetKind::Sprite,
            save_as,
            self.document.encode().map_err(|error| error.to_string()),
        );
    }
}

struct State {
    selected_color: u8,
    canvas: ui::canvas::CanvasView,
}

impl Default for State {
    fn default() -> Self {
        Self {
            selected_color: 1,
            canvas: ui::canvas::CanvasView::default(),
        }
    }
}

fn tools(ui: &mut Ui, state: &mut State) -> bool {
    if let Some(color) = ui::palette::show(ui, state.selected_color) {
        state.selected_color = color;
    }
    false
}

fn canvas(ui: &mut Ui, document: &mut SpriteDocument, state: &mut State) -> bool {
    let edit = ui::canvas::show_pixels(
        ui,
        document.pixels.pixels(),
        document.size,
        state.selected_color.max(1),
        PixelMode::Indexed,
        &mut state.canvas,
    );
    apply_pixel_edit(document.pixels.pixels_mut(), document.size, edit)
}
