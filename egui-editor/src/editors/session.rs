use crate::{document::AssetKind, file_io};
use std::path::PathBuf;

/// Per-file state shared by editor screens. Asset documents remain owned by
/// their concrete editor screen, so editing never has to unpack an asset enum.
pub struct FileState {
    pub path: PathBuf,
    pub linked_tileset_path: Option<PathBuf>,
    pub dirty: bool,
    pub status: String,
}

impl FileState {
    pub fn new(path: PathBuf, linked_tileset_path: Option<PathBuf>) -> Self {
        Self {
            path,
            linked_tileset_path,
            dirty: false,
            status: "Ready".into(),
        }
    }

    pub fn title(&self) -> &str {
        file_io::file_name(Some(&self.path))
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
        self.status = "Unsaved changes".into();
    }

    pub fn save(&mut self, kind: AssetKind, save_as: bool, bytes: Result<Vec<u8>, String>) {
        let bytes = match bytes {
            Ok(bytes) => bytes,
            Err(error) => {
                self.status = error;
                return;
            }
        };
        let path = if save_as {
            file_io::pick_save_path(kind, self.title())
        } else {
            Some(self.path.clone())
        };
        let Some(path) = path else { return };
        match file_io::write_bytes(&path, &bytes) {
            Ok(()) => {
                self.path = path;
                self.dirty = false;
                self.status = "Saved".into();
            }
            Err(error) => self.status = error,
        }
    }
}
