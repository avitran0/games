mod animated_sprite;
mod font;
mod session;
mod sprite;
mod tilemap;
mod tileset;

use crate::document::AssetDocument;
use crate::file_io::LoadedAsset;
use crate::ui::canvas::{PixelAction, PixelEdit, PixelRect};
use api::glam::UVec2;
use eframe::egui::Ui;

pub use session::FileState;

pub(super) fn apply_pixel_action(
    pixels: &mut [u8],
    size: UVec2,
    action: Option<PixelAction>,
    selected_color: &mut u8,
) -> bool {
    match action {
        Some(PixelAction::PickColor(color)) => {
            *selected_color = color;
            false
        }
        Some(PixelAction::Paint(edit)) => apply_pixel_edit(pixels, size, Some(edit)),
        Some(PixelAction::MoveSelection { from, to }) => {
            move_selection(pixels, size, from, to)
        }
        None => false,
    }
}

fn move_selection(pixels: &mut [u8], size: UVec2, from: PixelRect, to: PixelRect) -> bool {
    let width = size.x as usize;
    let height = size.y as usize;
    let Some(from_right) = from.x.checked_add(from.width) else {
        return false;
    };
    let Some(from_bottom) = from.y.checked_add(from.height) else {
        return false;
    };
    let Some(to_right) = to.x.checked_add(from.width) else {
        return false;
    };
    let Some(to_bottom) = to.y.checked_add(from.height) else {
        return false;
    };
    if from == to
        || from.width == 0
        || from.height == 0
        || to.width != from.width
        || to.height != from.height
        || from_right > width
        || from_bottom > height
        || to_right > width
        || to_bottom > height
        || pixels.len() != width * height
    {
        return false;
    }

    let mut selected = Vec::with_capacity(from.width * from.height);
    for y in 0..from.height {
        let start = (from.y + y) * width + from.x;
        selected.extend_from_slice(&pixels[start..start + from.width]);
    }

    let mut changed = false;
    for y in 0..from.height {
        let start = (from.y + y) * width + from.x;
        changed |= pixels[start..start + from.width].iter().any(|pixel| *pixel != 0);
        pixels[start..start + from.width].fill(0);
    }
    for y in 0..from.height {
        for x in 0..from.width {
            let value = selected[y * from.width + x];
            if value != 0 {
                let pixel = &mut pixels[(to.y + y) * width + to.x + x];
                changed |= *pixel != value;
                *pixel = value;
            }
        }
    }
    changed
}

pub(super) fn apply_pixel_edit(pixels: &mut [u8], size: UVec2, edit: Option<PixelEdit>) -> bool {
    let Some(edit) = edit else { return false };
    let index = edit.y * size.x as usize + edit.x;
    let Some(pixel) = pixels.get_mut(index) else {
        return false;
    };
    if *pixel == edit.value {
        return false;
    }
    *pixel = edit.value;
    true
}

trait Editor {
    fn file(&self) -> &FileState;
    fn file_mut(&mut self) -> &mut FileState;
    fn save(&mut self, save_as: bool);
    fn tools(&mut self, ui: &mut Ui) -> bool;
    fn canvas_controls(&mut self, ui: &mut Ui);
    fn canvas(&mut self, ui: &mut Ui) -> bool;

    fn dirty(&self) -> bool {
        self.file().dirty
    }

    fn mark_dirty(&mut self) {
        self.file_mut().mark_dirty();
    }

    fn status_detail(&self) -> Option<String> {
        None
    }

    fn has_toolbar(&self) -> bool {
        false
    }

    fn toolbar(&mut self, _ui: &mut Ui) -> bool {
        false
    }

    fn has_timeline(&self) -> bool {
        false
    }

    fn timeline(&mut self, _ui: &mut Ui) -> bool {
        false
    }

    fn has_tile_strip(&self) -> bool {
        false
    }

    fn tile_strip(&mut self, _ui: &mut Ui) -> bool {
        false
    }
}

macro_rules! impl_editor {
    ($screen:ty, $($extra:item),* $(,)?) => {
        impl Editor for $screen {
            fn file(&self) -> &FileState { &self.file }
            fn file_mut(&mut self) -> &mut FileState { &mut self.file }
            fn save(&mut self, save_as: bool) { <$screen>::save(self, save_as); }
            fn tools(&mut self, ui: &mut Ui) -> bool { <$screen>::tools(self, ui) }
            fn canvas_controls(&mut self, ui: &mut Ui) { <$screen>::canvas_controls(self, ui); }
            fn canvas(&mut self, ui: &mut Ui) -> bool { <$screen>::canvas(self, ui) }
            $($extra)*
        }
    };
}

impl_editor!(
    sprite::Screen,
    fn status_detail(&self) -> Option<String> {
        Some(self.info())
    }
);

impl_editor!(
    animated_sprite::Screen,
    fn has_toolbar(&self) -> bool {
        true
    },
    fn toolbar(&mut self, ui: &mut Ui) -> bool {
        animated_sprite::Screen::toolbar(self, ui)
    },
    fn has_timeline(&self) -> bool {
        true
    },
    fn timeline(&mut self, ui: &mut Ui) -> bool {
        animated_sprite::Screen::timeline(self, ui)
    }
);

impl_editor!(
    font::Screen,
    fn has_toolbar(&self) -> bool {
        true
    },
    fn toolbar(&mut self, ui: &mut Ui) -> bool {
        font::Screen::toolbar(self, ui)
    }
);

impl_editor!(
    tileset::Screen,
    fn has_toolbar(&self) -> bool {
        true
    },
    fn toolbar(&mut self, ui: &mut Ui) -> bool {
        tileset::Screen::toolbar(self, ui)
    },
    fn has_tile_strip(&self) -> bool {
        true
    },
    fn tile_strip(&mut self, ui: &mut Ui) -> bool {
        tileset::Screen::tile_strip(self, ui)
    }
);

impl_editor!(
    tilemap::Screen,
    fn dirty(&self) -> bool {
        tilemap::Screen::dirty(self)
    },
    fn mark_dirty(&mut self) {
        tilemap::Screen::mark_dirty(self);
    },
    fn has_toolbar(&self) -> bool {
        true
    },
    fn toolbar(&mut self, ui: &mut Ui) -> bool {
        tilemap::Screen::toolbar(self, ui)
    },
    fn has_tile_strip(&self) -> bool {
        true
    },
    fn tile_strip(&mut self, ui: &mut Ui) -> bool {
        tilemap::Screen::tile_strip(self, ui)
    }
);

pub struct EditorScreen(Box<dyn Editor>);

impl EditorScreen {
    pub fn new(loaded: LoadedAsset) -> Self {
        let file = FileState::new(loaded.path, loaded.linked_tileset_path);
        Self(match loaded.document {
            AssetDocument::Sprite(document) => Box::new(sprite::Screen::new(document, file)),
            AssetDocument::AnimatedSprite(document) => {
                Box::new(animated_sprite::Screen::new(document, file))
            }
            AssetDocument::Font(document) => Box::new(font::Screen::new(document, file)),
            AssetDocument::Tileset(document) => Box::new(tileset::Screen::new(document, file)),
            AssetDocument::Tilemap { map, tileset } => {
                Box::new(tilemap::Screen::new(map, tileset, file))
            }
        })
    }

    pub fn title(&self) -> &str {
        self.0.file().title()
    }

    pub fn has_toolbar(&self) -> bool {
        self.0.has_toolbar()
    }

    pub fn status_detail(&self) -> Option<String> {
        self.0.status_detail()
    }

    pub fn dirty(&self) -> bool {
        self.0.dirty()
    }

    pub fn status(&self) -> &str {
        &self.0.file().status
    }

    pub fn mark_dirty(&mut self) {
        self.0.mark_dirty();
    }

    pub fn set_status(&mut self, status: String) {
        self.0.file_mut().status = status;
    }

    pub fn save(&mut self, save_as: bool) {
        self.0.save(save_as);
    }

    pub fn toolbar(&mut self, ui: &mut Ui) -> bool {
        self.0.toolbar(ui)
    }

    pub fn tools(&mut self, ui: &mut Ui) -> bool {
        self.0.tools(ui)
    }

    pub fn canvas_controls(&mut self, ui: &mut Ui) {
        self.0.canvas_controls(ui);
    }

    pub fn has_timeline(&self) -> bool {
        self.0.has_timeline()
    }

    pub fn has_tile_strip(&self) -> bool {
        self.0.has_tile_strip()
    }

    pub fn tile_strip(&mut self, ui: &mut Ui) -> bool {
        self.0.tile_strip(ui)
    }

    pub fn timeline(&mut self, ui: &mut Ui) -> bool {
        self.0.timeline(ui)
    }

    pub fn canvas(&mut self, ui: &mut Ui) -> bool {
        self.0.canvas(ui)
    }
}
