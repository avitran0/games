use super::{AssetSpec, dimensions};
use crate::file_io;
use api::formats::TilesetDocument;
use api::glam::uvec2;
use eframe::egui::{self, Ui};
use std::path::PathBuf;

pub struct Screen {
    width: u32,
    height: u32,
    tileset: Option<(TilesetDocument, PathBuf)>,
    error: Option<String>,
}

impl Default for Screen {
    fn default() -> Self {
        Self {
            width: 16,
            height: 16,
            tileset: None,
            error: None,
        }
    }
}

impl Screen {
    pub fn show(&mut self, ui: &mut Ui) -> Option<AssetSpec> {
        ui.heading("Tilemap");
        ui.label("Create a cell grid linked to a separately stored tileset.");
        ui.add_space(12.0);
        dimensions::cell_size(ui, &mut self.width, &mut self.height);
        ui.add_space(12.0);
        if ui.button("Choose tileset...").clicked()
            && let Some(path) = file_io::pick_tileset_path()
        {
            match file_io::load_tileset(&path) {
                Ok(tileset) => {
                    self.tileset = Some((tileset, path));
                    self.error = None;
                }
                Err(error) => self.error = Some(error),
            }
        }
        if let Some((tileset, path)) = &self.tileset {
            ui.label(format!(
                "{} - {} tiles - {}",
                path.file_name().unwrap_or_default().to_string_lossy(),
                tileset.tiles.len(),
                tileset.id
            ));
        } else {
            ui.label("Choose the tileset this map will reference.");
        }
        if let Some(error) = &self.error {
            ui.colored_label(ui.visuals().error_fg_color, error);
        }
        ui.add_space(16.0);
        if ui
            .add_enabled(self.tileset.is_some(), egui::Button::new("Create tilemap"))
            .clicked()
            && let Some((tileset, tileset_path)) = &self.tileset
        {
            return Some(AssetSpec::Tilemap {
                size: uvec2(self.width, self.height),
                tileset: tileset.clone(),
                tileset_path: tileset_path.clone(),
            });
        }
        None
    }
}
