mod animated_sprite;
mod create;
mod dimensions;
mod font;
mod home;
mod sprite;
mod tilemap;
mod tileset;

use crate::document::AssetKind;
use api::glam::UVec2;
use eframe::egui::{self, Ui};
use formats::TilesetDocument;
use std::path::PathBuf;

#[derive(Clone)]
pub enum AssetSpec {
    Sprite(UVec2),
    AnimatedSprite(UVec2),
    Font(u16),
    Tileset(UVec2),
    Tilemap {
        size: UVec2,
        tileset: TilesetDocument,
        tileset_path: PathBuf,
    },
}

pub enum Action {
    None,
    Open,
    StartCreate(AssetKind),
    Create(AssetSpec),
}

trait CreatePage {
    fn show(&mut self, ui: &mut Ui) -> Option<AssetSpec>;
}

macro_rules! impl_create_page {
    ($screen:ty) => {
        impl CreatePage for $screen {
            fn show(&mut self, ui: &mut Ui) -> Option<AssetSpec> {
                <$screen>::show(self, ui)
            }
        }
    };
}

impl_create_page!(sprite::Screen);
impl_create_page!(animated_sprite::Screen);
impl_create_page!(font::Screen);
impl_create_page!(tileset::Screen);
impl_create_page!(tilemap::Screen);

#[derive(Default)]
pub struct SetupScreen {
    page: Option<(AssetKind, Box<dyn CreatePage>)>,
    error: Option<String>,
}

impl SetupScreen {
    pub fn show(&mut self, ui: &mut Ui) -> Action {
        if self.page.is_none() {
            let mut action = Action::None;
            egui::CentralPanel::default().show(ui, |ui| {
                action = home::show(ui);
                if let Some(error) = &self.error {
                    ui.separator();
                    ui.colored_label(ui.visuals().error_fg_color, error);
                }
            });
            if let Action::StartCreate(kind) = action {
                self.open_create_screen(kind);
                return Action::None;
            }
            return action;
        }

        let kind = self.page.as_ref().unwrap().0;
        let mut back = false;
        egui::Panel::top("setup-header").show(ui, |ui| {
            ui.horizontal(|ui| {
                back = ui.button("Back").clicked();
                ui.separator();
                ui.heading(format!("New {}", kind.title().to_lowercase()));
            });
        });
        if back {
            self.page = None;
            return Action::None;
        }

        let mut action = Action::None;
        egui::CentralPanel::default().show(ui, |ui| {
            action = self
                .page
                .as_mut()
                .unwrap()
                .1
                .show(ui)
                .map_or(Action::None, Action::Create);
            if let Some(error) = &self.error {
                ui.add_space(12.0);
                ui.colored_label(ui.visuals().error_fg_color, error);
            }
        });
        action
    }

    pub fn set_error(&mut self, error: impl Into<String>) {
        self.error = Some(error.into());
    }

    pub fn go_home(&mut self) {
        self.page = None;
        self.error = None;
    }

    pub fn open_create_screen(&mut self, kind: AssetKind) {
        self.error = None;
        let page: Box<dyn CreatePage> = match kind {
            AssetKind::Sprite => Box::new(sprite::Screen::default()),
            AssetKind::AnimatedSprite => Box::new(animated_sprite::Screen::default()),
            AssetKind::Font => Box::new(font::Screen::default()),
            AssetKind::Tileset => Box::new(tileset::Screen::default()),
            AssetKind::Tilemap => Box::new(tilemap::Screen::default()),
        };
        self.page = Some((kind, page));
    }
}

pub fn create_asset(spec: AssetSpec) -> Result<Option<crate::file_io::LoadedAsset>, String> {
    create::create(spec)
}
