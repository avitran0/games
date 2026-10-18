use crate::{
    document::AssetKind,
    editors::{FileState, apply_pixel_action},
    ui::{self, canvas::PixelMode},
};
use eframe::egui::{self, Ui};
use formats::{NamedSprite, SpriteDocument, SpriteSheetDocument};

pub struct Screen {
    pub file: FileState,
    document: SpriteSheetDocument,
    state: State,
}

impl Screen {
    pub fn new(document: SpriteSheetDocument, file: FileState) -> Self {
        Self {
            file,
            document,
            state: State::default(),
        }
    }

    pub fn toolbar(&mut self, ui: &mut Ui) -> bool {
        let count = self.document.sprites.len();
        let selected = self.state.selected;
        let named = &mut self.document.sprites[selected];
        ui.horizontal(|ui| {
            ui.label("Name");
            ui.add(egui::TextEdit::singleline(&mut named.name).desired_width(180.0));
            ui.separator();
            ui.label(format!(
                "{} x {} px · sprite {} of {}",
                named.sprite.size.x,
                named.sprite.size.y,
                self.state.selected + 1,
                count
            ));
        })
        .response
        .changed()
    }

    pub fn tools(&mut self, ui: &mut Ui) -> bool {
        let mut changed = false;
        ui.heading("Sprites");
        for (index, named) in self.document.sprites.iter().enumerate() {
            if ui
                .selectable_value(&mut self.state.selected, index, &named.name)
                .clicked()
            {
                self.state.reset_canvas();
            }
        }
        ui.horizontal(|ui| {
            if ui.button("Add").clicked() {
                let size = self.document.sprites[self.state.selected].sprite.size;
                let mut name_index = self.document.sprites.len() + 1;
                let name = loop {
                    let candidate = format!("sprite_{name_index}");
                    if !self
                        .document
                        .sprites
                        .iter()
                        .any(|sprite| sprite.name == candidate)
                    {
                        break candidate;
                    }
                    name_index += 1;
                };
                if let Ok(sprite) = SpriteDocument::new(size) {
                    self.document.sprites.push(NamedSprite { name, sprite });
                    self.state.selected = self.document.sprites.len() - 1;
                    self.state.reset_canvas();
                    changed = true;
                }
            }
            if ui.button("Duplicate").clicked() {
                let mut copy = self.document.sprites[self.state.selected].clone();
                let base = format!("{}_copy", copy.name);
                copy.name = base.clone();
                let mut suffix = 2;
                while self
                    .document
                    .sprites
                    .iter()
                    .any(|sprite| sprite.name == copy.name)
                {
                    copy.name = format!("{base}{suffix}");
                    suffix += 1;
                }
                self.document.sprites.push(copy);
                self.state.selected = self.document.sprites.len() - 1;
                self.state.reset_canvas();
                changed = true;
            }
            if ui
                .add_enabled(self.document.sprites.len() > 1, egui::Button::new("Remove"))
                .clicked()
            {
                self.document.sprites.remove(self.state.selected);
                self.state.selected = self.state.selected.min(self.document.sprites.len() - 1);
                self.state.reset_canvas();
                changed = true;
            }
        });
        ui.separator();
        ui.label("Palette index");
        if let Some(color) = ui::palette::show(ui, self.state.selected_color) {
            self.state.selected_color = color;
        }
        changed
    }

    pub fn canvas_controls(&mut self, ui: &mut Ui) {
        ui::canvas::controls(ui, &mut self.state.canvas);
    }

    pub fn canvas(&mut self, ui: &mut Ui) -> bool {
        let named = &mut self.document.sprites[self.state.selected];
        let edit = ui::canvas::show_pixels_with_selection(
            ui,
            named.sprite.pixels.pixels(),
            named.sprite.size,
            self.state.selected_color,
            PixelMode::Indexed,
            &mut self.state.selection,
            &mut self.state.canvas,
        );
        apply_pixel_action(
            named.sprite.pixels.pixels_mut(),
            named.sprite.size,
            edit,
            &mut self.state.selected_color,
        )
    }

    pub fn info(&self) -> String {
        format!(
            "Sprite sheet: {} named sprites",
            self.document.sprites.len()
        )
    }

    pub fn save(&mut self, save_as: bool) {
        self.file.save(
            AssetKind::SpriteSheet,
            save_as,
            self.document.encode().map_err(|error| error.to_string()),
        );
    }
}

struct State {
    selected: usize,
    selected_color: u8,
    selection: ui::canvas::PixelSelection,
    canvas: ui::canvas::CanvasView,
}

impl State {
    fn reset_canvas(&mut self) {
        self.canvas = ui::canvas::CanvasView::default();
        self.selection = ui::canvas::PixelSelection::default();
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            selected: 0,
            selected_color: 1,
            selection: ui::canvas::PixelSelection::default(),
            canvas: ui::canvas::CanvasView::default(),
        }
    }
}
