use crate::{editors::EditorScreen, file_io, setup, ui::menu_bar};
use eframe::egui;

pub struct App {
    view: View,
    setup_screen: setup::SetupScreen,
}

enum View {
    Setup,
    Editor(EditorScreen),
}

impl Default for App {
    fn default() -> Self {
        Self {
            view: View::Setup,
            setup_screen: setup::SetupScreen::default(),
        }
    }
}

impl App {
    fn show_editor(ui: &mut egui::Ui, editor: &mut EditorScreen) -> Option<menu_bar::Action> {
        let mut action = None;

        egui::Panel::top("menu-bar").show(ui, |ui| {
            action = menu_bar::show(ui, true);
        });

        if action.is_some() {
            return action;
        }

        if editor.has_toolbar() {
            egui::Panel::top("document-toolbar").show(ui, |ui| {
                if editor.toolbar(ui) {
                    editor.mark_dirty();
                }
            });
        }

        let status_detail = editor.status_detail();
        egui::Panel::bottom("status-bar").show(ui, |ui| {
            crate::ui::status::show(
                ui,
                editor.dirty(),
                editor.status(),
                status_detail.as_deref(),
            );
        });
        if editor.has_timeline() {
            egui::Panel::bottom("animation-timeline")
                .default_size(190.0)
                .min_size(120.0)
                .max_size(360.0)
                .resizable(true)
                .show(ui, |ui| {
                    if editor.timeline(ui) {
                        editor.mark_dirty();
                    }
                });
        } else if editor.has_tile_strip() {
            egui::Panel::bottom("tileset-overview-panel")
                .default_size(130.0)
                .min_size(112.0)
                .max_size(220.0)
                .resizable(true)
                .show(ui, |ui| {
                    if editor.tile_strip(ui) {
                        editor.mark_dirty();
                    }
                });
        } else if editor.has_preview_panel() {
            egui::Panel::bottom("font-preview-panel")
                .default_size(150.0)
                .min_size(100.0)
                .max_size(280.0)
                .resizable(true)
                .show(ui, |ui| editor.preview_panel(ui));
        }
        egui::Panel::left("asset-tools")
            .default_size(178.0)
            .min_size(154.0)
            .max_size(260.0)
            .resizable(true)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    editor.canvas_controls(ui);
                    if editor.tools(ui) {
                        editor.mark_dirty();
                    }
                });
            });
        egui::CentralPanel::default().show(ui, |ui| {
            if editor.canvas(ui) {
                editor.mark_dirty();
            }
        });
        ui.ctx()
            .send_viewport_cmd(egui::ViewportCommand::Title(if editor.dirty() {
                format!("{} * - Pixel Sprite Studio", editor.title())
            } else {
                format!("{} - Pixel Sprite Studio", editor.title())
            }));
        None
    }

    fn can_discard(editor: &EditorScreen) -> bool {
        !editor.dirty()
            || rfd::MessageDialog::new()
                .set_title("Unsaved changes")
                .set_description("Discard unsaved changes?")
                .set_buttons(rfd::MessageButtons::YesNo)
                .show()
                == rfd::MessageDialogResult::Yes
    }

    fn setup_action(&mut self, action: setup::Action) {
        match action {
            setup::Action::None | setup::Action::StartCreate(_) => {}
            setup::Action::Open => match file_io::open() {
                Ok(Some(loaded)) => self.view = View::Editor(EditorScreen::new(loaded)),
                Ok(None) => {}
                Err(error) => self.setup_screen.set_error(error),
            },
            setup::Action::Create(spec) => match setup::create_asset(spec) {
                Ok(Some(loaded)) => self.view = View::Editor(EditorScreen::new(loaded)),
                Ok(None) => {}
                Err(error) => self.setup_screen.set_error(error),
            },
        }
    }

    fn editor_action(
        &mut self,
        action: menu_bar::Action,
        editor: &mut EditorScreen,
        ctx: &egui::Context,
    ) -> bool {
        match action {
            menu_bar::Action::New(kind) => {
                if Self::can_discard(editor) {
                    self.setup_screen.open_create_screen(kind);
                    return false;
                }
            }
            menu_bar::Action::Open => {
                if Self::can_discard(editor) {
                    match file_io::open() {
                        Ok(Some(loaded)) => {
                            self.view = View::Editor(EditorScreen::new(loaded));
                            return false;
                        }
                        Ok(None) => {}
                        Err(error) => editor.set_status(error),
                    }
                }
            }
            menu_bar::Action::Save => editor.save(false),
            menu_bar::Action::SaveAs => editor.save(true),
            menu_bar::Action::Back => {
                if Self::can_discard(editor) {
                    self.setup_screen.go_home();
                    return false;
                }
            }
            menu_bar::Action::Quit => {
                if Self::can_discard(editor) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
        true
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let view = std::mem::replace(&mut self.view, View::Setup);
        match view {
            View::Setup => {
                let action = self.setup_screen.show(ui);
                self.setup_action(action);
            }
            View::Editor(mut editor) => {
                let action = Self::show_editor(ui, &mut editor);
                let keep_editor =
                    action.is_none_or(|action| self.editor_action(action, &mut editor, ui.ctx()));
                if keep_editor {
                    self.view = View::Editor(editor);
                }
            }
        }
    }
}
