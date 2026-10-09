use crate::{
    document::AssetKind,
    editors::{FileState, apply_pixel_action},
    ui::{self, canvas::PixelMode},
};
use eframe::egui::Ui;
use formats::{SpriteDocument, TilesetDocument};

pub struct Screen {
    pub file: FileState,
    document: TilesetDocument,
    state: State,
}

impl Screen {
    pub fn new(document: TilesetDocument, file: FileState) -> Self {
        Self {
            file,
            document,
            state: State::default(),
        }
    }

    pub fn toolbar(&mut self, ui: &mut Ui) -> bool {
        toolbar(ui, &mut self.document, &mut self.state)
    }

    pub fn tools(&mut self, ui: &mut Ui) -> bool {
        tools(ui, &mut self.document, &mut self.state)
    }

    pub fn canvas_controls(&mut self, ui: &mut Ui) {
        ui::canvas::controls(ui, &mut self.state.canvas);
    }

    pub fn tile_strip(&mut self, ui: &mut Ui) -> bool {
        let action = ui::tile_strip::show(ui, &self.document, Some(self.state.selected_tile), true);
        match action {
            Some(ui::tile_strip::Action::Select(index)) => {
                self.state.selected_tile = index;
                false
            }
            Some(ui::tile_strip::Action::Add) => {
                if let Ok(blank) = SpriteDocument::new(self.document.tile_size) {
                    self.document.tiles.push(blank.pixels);
                    self.state.selected_tile = self.document.tiles.len() - 1;
                    true
                } else {
                    false
                }
            }
            Some(ui::tile_strip::Action::Duplicate) => {
                let copy = self.document.tiles[self.state.selected_tile].clone();
                self.document.tiles.push(copy);
                self.state.selected_tile = self.document.tiles.len() - 1;
                true
            }
            Some(ui::tile_strip::Action::Remove) if self.document.tiles.len() > 1 => {
                self.document.tiles.remove(self.state.selected_tile);
                self.state.selected_tile =
                    self.state.selected_tile.min(self.document.tiles.len() - 1);
                true
            }
            Some(ui::tile_strip::Action::MoveLeft) if self.state.selected_tile > 0 => {
                self.document
                    .tiles
                    .swap(self.state.selected_tile, self.state.selected_tile - 1);
                self.state.selected_tile -= 1;
                true
            }
            Some(ui::tile_strip::Action::MoveRight)
                if self.state.selected_tile + 1 < self.document.tiles.len() =>
            {
                self.document
                    .tiles
                    .swap(self.state.selected_tile, self.state.selected_tile + 1);
                self.state.selected_tile += 1;
                true
            }
            Some(ui::tile_strip::Action::Remove)
            | Some(ui::tile_strip::Action::MoveLeft)
            | Some(ui::tile_strip::Action::MoveRight)
            | None => false,
        }
    }

    pub fn canvas(&mut self, ui: &mut Ui) -> bool {
        canvas(ui, &mut self.document, &mut self.state)
    }

    pub fn save(&mut self, save_as: bool) {
        self.file.save(
            AssetKind::Tileset,
            save_as,
            self.document.encode().map_err(|error| error.to_string()),
        );
    }
}

struct State {
    selected_tile: usize,
    selected_color: u8,
    canvas: ui::canvas::CanvasView,
}

impl Default for State {
    fn default() -> Self {
        Self {
            selected_tile: 0,
            selected_color: 1,
            canvas: ui::canvas::CanvasView::default(),
        }
    }
}

fn toolbar(ui: &mut Ui, document: &mut TilesetDocument, state: &mut State) -> bool {
    let count = document.tiles.len();
    state.selected_tile = state.selected_tile.min(count.saturating_sub(1));
    ui.label(format!(
        "Tile {} of {}: {} x {} px",
        state.selected_tile + 1,
        count,
        document.tile_size.x,
        document.tile_size.y
    ));
    false
}

fn tools(ui: &mut Ui, _document: &mut TilesetDocument, state: &mut State) -> bool {
    if let Some(color) = ui::palette::show(ui, state.selected_color) {
        state.selected_color = color;
    }
    false
}

fn canvas(ui: &mut Ui, document: &mut TilesetDocument, state: &mut State) -> bool {
    let frame = &mut document.tiles[state.selected_tile];
    let edit = ui::canvas::show_pixels(
        ui,
        frame.pixels(),
        document.tile_size,
        state.selected_color,
        PixelMode::Indexed,
        &mut state.canvas,
    );
    apply_pixel_action(
        frame.pixels_mut(),
        document.tile_size,
        edit,
        &mut state.selected_color,
    )
}
