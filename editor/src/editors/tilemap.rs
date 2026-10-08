use crate::{
    document::AssetKind,
    editors::{FileState, apply_pixel_action},
    file_io,
    ui::{self, canvas::PixelMode},
};
use api::formats::{SpriteDocument, TilemapDocument, TilesetDocument};
use eframe::egui::Ui;

pub struct Screen {
    pub file: FileState,
    map: TilemapDocument,
    tileset: TilesetDocument,
    state: State,
    tileset_dirty: bool,
}

impl Screen {
    pub fn new(map: TilemapDocument, tileset: TilesetDocument, file: FileState) -> Self {
        Self {
            file,
            map,
            tileset,
            state: State::default(),
            tileset_dirty: false,
        }
    }

    pub fn dirty(&self) -> bool {
        self.file.dirty || self.tileset_dirty
    }

    pub fn mark_dirty(&mut self) {
        match self.state.mode {
            EditMode::Map => self.file.mark_dirty(),
            EditMode::Tileset => {
                self.tileset_dirty = true;
                self.file.status = "Unsaved changes".into();
            }
        }
    }

    pub fn toolbar(&mut self, ui: &mut Ui) -> bool {
        toolbar(ui, &self.map, &self.tileset, &mut self.state)
    }

    pub fn tools(&mut self, ui: &mut Ui) -> bool {
        match self.state.mode {
            EditMode::Map => tools_map(ui, &self.tileset, &mut self.state),
            EditMode::Tileset => tools_tileset(ui, &mut self.state),
        }
    }

    pub fn canvas_controls(&mut self, ui: &mut Ui) {
        let view = match self.state.mode {
            EditMode::Map => &mut self.state.map_canvas,
            EditMode::Tileset => &mut self.state.tile_canvas,
        };
        ui::canvas::controls(ui, view);
    }

    pub fn tile_strip(&mut self, ui: &mut Ui) -> bool {
        let selected = match self.state.mode {
            EditMode::Map => self
                .state
                .selected_tile
                .checked_sub(1)
                .map(|index| index as usize),
            EditMode::Tileset => Some(self.state.tile_index),
        };
        let action = ui::tile_strip::show(
            ui,
            &self.tileset,
            selected,
            self.state.mode == EditMode::Tileset,
        );
        match action {
            Some(ui::tile_strip::Action::Select(index)) => {
                self.state.tile_index = index;
                self.state.selected_tile = (index + 1) as u32;
                false
            }
            Some(ui::tile_strip::Action::Add) => {
                if let Ok(blank) = SpriteDocument::new(self.tileset.tile_size) {
                    self.tileset.tiles.push(blank.pixels);
                    self.state.tile_index = self.tileset.tiles.len() - 1;
                    self.state.selected_tile = (self.state.tile_index + 1) as u32;
                    true
                } else {
                    false
                }
            }
            Some(ui::tile_strip::Action::Duplicate) => {
                let copy = self.tileset.tiles[self.state.tile_index].clone();
                self.tileset.tiles.push(copy);
                self.state.tile_index = self.tileset.tiles.len() - 1;
                self.state.selected_tile = (self.state.tile_index + 1) as u32;
                true
            }
            Some(ui::tile_strip::Action::Remove) if self.tileset.tiles.len() > 1 => {
                self.tileset.tiles.remove(self.state.tile_index);
                self.state.tile_index = self.state.tile_index.min(self.tileset.tiles.len() - 1);
                self.state.selected_tile = (self.state.tile_index + 1) as u32;
                true
            }
            Some(ui::tile_strip::Action::Remove) | None => false,
        }
    }

    pub fn canvas(&mut self, ui: &mut Ui) -> bool {
        match self.state.mode {
            EditMode::Map => canvas_map(ui, &mut self.map, &self.tileset, &mut self.state),
            EditMode::Tileset => canvas_tileset(ui, &mut self.tileset, &mut self.state),
        }
    }

    pub fn save(&mut self, save_as: bool) {
        let map_path = if save_as {
            file_io::pick_save_path(AssetKind::Tilemap, self.file.title())
        } else {
            Some(self.file.path.clone())
        };
        let Some(map_path) = map_path else {
            return;
        };

        let map_bytes = match self.map.encode() {
            Ok(bytes) => bytes,
            Err(error) => {
                self.file.status = error.to_string();
                return;
            }
        };
        if self.tileset_dirty {
            let tileset_bytes = match self.tileset.encode() {
                Ok(bytes) => bytes,
                Err(error) => {
                    self.file.status = error.to_string();
                    return;
                }
            };
            let tileset_path = self
                .file
                .linked_tileset_path
                .clone()
                .or_else(|| file_io::pick_save_path(AssetKind::Tileset, "tileset.pxt"));
            let Some(tileset_path) = tileset_path else {
                return;
            };
            if let Err(error) = file_io::write_bytes(&tileset_path, &tileset_bytes) {
                self.file.status = error;
                return;
            }
            self.file.linked_tileset_path = Some(tileset_path);
            self.tileset_dirty = false;
        }

        if let Err(error) = file_io::write_bytes(&map_path, &map_bytes) {
            self.file.status = error;
            return;
        }
        self.file.path = map_path;
        self.file.dirty = false;
        self.file.status = "Saved".into();
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum EditMode {
    #[default]
    Map,
    Tileset,
}

struct State {
    mode: EditMode,
    selected_tile: u32,
    selected_color: u8,
    tile_index: usize,
    flip_x: bool,
    flip_y: bool,
    flip_diagonal: bool,
    map_canvas: ui::canvas::CanvasView,
    tile_canvas: ui::canvas::CanvasView,
}

impl Default for State {
    fn default() -> Self {
        Self {
            mode: EditMode::Map,
            selected_tile: 0,
            selected_color: 1,
            tile_index: 0,
            flip_x: false,
            flip_y: false,
            flip_diagonal: false,
            map_canvas: ui::canvas::CanvasView::default(),
            tile_canvas: ui::canvas::CanvasView::default(),
        }
    }
}

fn toolbar(
    ui: &mut Ui,
    map: &TilemapDocument,
    tileset: &TilesetDocument,
    state: &mut State,
) -> bool {
    let tile_count = tileset.tiles.len();
    state.selected_tile = state.selected_tile.min(tile_count as u32);
    state.tile_index = state.tile_index.min(tile_count.saturating_sub(1));

    ui.horizontal(|ui| {
        if ui
            .selectable_label(state.mode == EditMode::Map, "Tilemap")
            .clicked()
            && state.mode != EditMode::Map
        {
            state.mode = EditMode::Map;
            state.selected_tile = (state.tile_index + 1) as u32;
        }
        if ui
            .selectable_label(state.mode == EditMode::Tileset, "Tileset")
            .clicked()
            && state.mode != EditMode::Tileset
        {
            state.mode = EditMode::Tileset;
            state.tile_index = state
                .selected_tile
                .saturating_sub(1)
                .min(tile_count.saturating_sub(1) as u32) as usize;
        }
        ui.separator();

        match state.mode {
            EditMode::Map => {
                ui.label(format!(
                    "{} x {} cells, {} tiles",
                    map.size.x, map.size.y, tile_count
                ));
                ui.separator();
                ui.checkbox(&mut state.flip_x, "Flip X");
                ui.checkbox(&mut state.flip_y, "Flip Y");
                ui.checkbox(&mut state.flip_diagonal, "Transpose");
            }
            EditMode::Tileset => {
                ui.label(format!(
                    "Tile {} of {}: {} x {} px",
                    state.tile_index + 1,
                    tile_count,
                    tileset.tile_size.x,
                    tileset.tile_size.y
                ));
            }
        }
    });
    false
}

fn tools_map(ui: &mut Ui, tileset: &TilesetDocument, state: &mut State) -> bool {
    if let Some(tile_id) = ui::tile_palette::show(ui, tileset, state.selected_tile) {
        state.selected_tile = tile_id;
    }
    false
}

fn tools_tileset(ui: &mut Ui, state: &mut State) -> bool {
    if let Some(color) = ui::palette::show(ui, state.selected_color) {
        state.selected_color = color;
    }
    false
}

fn canvas_map(
    ui: &mut Ui,
    map: &mut TilemapDocument,
    tileset: &TilesetDocument,
    state: &mut State,
) -> bool {
    let edit: Option<ui::canvas::CellEdit> = ui::canvas::show_tilemap(
        ui,
        map,
        tileset,
        ui::canvas::TilePlacement {
            selected_id: state.selected_tile,
            flip_x: state.flip_x,
            flip_y: state.flip_y,
            flip_diagonal: state.flip_diagonal,
        },
        &mut state.map_canvas,
    );
    let Some(edit) = edit else { return false };
    let Some(cell) = map.cells.get_mut(edit.index) else {
        return false;
    };
    if *cell == edit.cell {
        return false;
    }
    *cell = edit.cell;
    true
}

fn canvas_tileset(ui: &mut Ui, tileset: &mut TilesetDocument, state: &mut State) -> bool {
    let Some(tile) = tileset.tiles.get_mut(state.tile_index) else {
        return false;
    };
    let edit = ui::canvas::show_pixels(
        ui,
        tile.pixels(),
        tileset.tile_size,
        state.selected_color,
        PixelMode::Indexed,
        &mut state.tile_canvas,
    );
    apply_pixel_action(
        tile.pixels_mut(),
        tileset.tile_size,
        edit,
        &mut state.selected_color,
    )
}
