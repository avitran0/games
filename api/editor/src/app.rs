use crate::{editors::EditorScreen, file_io, setup, ui::menu_bar};
use eframe::egui;

pub struct App {
    editors: Vec<EditorScreen>,
    active_editor: usize,
    showing_setup: bool,
    setup_screen: setup::SetupScreen,
}

impl Default for App {
    fn default() -> Self {
        Self {
            editors: Vec::new(),
            active_editor: 0,
            showing_setup: true,
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
                .default_size(200.0)
                .resizable(true)
                .show(ui, |ui| {
                    if editor.timeline(ui) {
                        editor.mark_dirty();
                    }
                });
        } else if editor.has_tile_strip() {
            egui::Panel::bottom("tileset-overview-panel")
                .default_size(150.0)
                .resizable(true)
                .show(ui, |ui| {
                    if editor.tile_strip(ui) {
                        editor.mark_dirty();
                    }
                });
        } else if editor.has_preview_panel() {
            egui::Panel::bottom("font-glyph-panel")
                .default_size(150.0)
                .resizable(true)
                .show(ui, |ui| {
                    if editor.preview_panel(ui) {
                        editor.mark_dirty();
                    }
                });
        }
        egui::Panel::left("asset-tools")
            .default_size(300.0)
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
                format!("{} * - Editor", editor.title())
            } else {
                format!("{} - Editor", editor.title())
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

    fn push_loaded(&mut self, loaded: file_io::LoadedAsset) {
        self.editors.push(EditorScreen::new(loaded));
        self.active_editor = self.editors.len() - 1;
        self.showing_setup = false;
    }

    fn setup_action(&mut self, action: setup::Action) {
        match action {
            setup::Action::None | setup::Action::StartCreate(_) => {}
            setup::Action::Open => match file_io::open() {
                Ok(Some(loaded)) => self.push_loaded(loaded),
                Ok(None) => {}
                Err(error) => self.setup_screen.set_error(error),
            },
            setup::Action::Create(spec) => match setup::create_asset(spec) {
                Ok(Some(loaded)) => self.push_loaded(loaded),
                Ok(None) => {}
                Err(error) => self.setup_screen.set_error(error),
            },
        }
    }

    fn editor_action(&mut self, action: menu_bar::Action, ctx: &egui::Context) {
        match action {
            menu_bar::Action::New(kind) => {
                self.setup_screen.open_create_screen(kind);
                self.showing_setup = true;
            }
            menu_bar::Action::Open => match file_io::open() {
                Ok(Some(loaded)) => self.push_loaded(loaded),
                Ok(None) => {}
                Err(error) => {
                    if let Some(editor) = self.editors.get_mut(self.active_editor) {
                        editor.set_status(error);
                    }
                }
            },
            menu_bar::Action::Save => {
                if let Some(editor) = self.editors.get_mut(self.active_editor) {
                    editor.save(false);
                }
            }
            menu_bar::Action::SaveAs => {
                if let Some(editor) = self.editors.get_mut(self.active_editor) {
                    editor.save(true);
                }
            }
            menu_bar::Action::Back => {
                self.setup_screen.go_home();
                self.showing_setup = true;
            }
            menu_bar::Action::Quit => {
                if self.editors.iter().all(Self::can_discard) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }

    fn show_tabs(&mut self, ui: &mut egui::Ui) {
        let mut close_tab = None;
        egui::Panel::top("editor-tabs").show(ui, |ui| {
            egui::ScrollArea::horizontal().show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (index, editor) in self.editors.iter().enumerate() {
                        if ui
                            .selectable_value(
                                &mut self.active_editor,
                                index,
                                format!(
                                    "{}{}",
                                    editor.title(),
                                    if editor.dirty() { " *" } else { "" }
                                ),
                            )
                            .clicked()
                        {
                            self.showing_setup = false;
                        }
                        if ui.small_button("×").on_hover_text("Close tab").clicked() {
                            close_tab = Some(index);
                        }
                        ui.add_space(4.0);
                    }
                    if ui
                        .button("+")
                        .on_hover_text("Open or create an asset")
                        .clicked()
                    {
                        self.showing_setup = true;
                        self.setup_screen.go_home();
                    }
                });
            });
        });

        if let Some(index) = close_tab
            && Self::can_discard(&self.editors[index])
        {
            self.editors.remove(index);
            if self.editors.is_empty() {
                self.active_editor = 0;
                self.showing_setup = true;
            } else {
                if index < self.active_editor {
                    self.active_editor -= 1;
                } else if self.active_editor >= self.editors.len() {
                    self.active_editor = self.editors.len() - 1;
                }
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if !self.editors.is_empty() {
            self.show_tabs(ui);
        }

        if self.showing_setup || self.editors.is_empty() {
            let action = self.setup_screen.show(ui);
            self.setup_action(action);
        } else {
            let active = self.active_editor.min(self.editors.len() - 1);
            let action = Self::show_editor(ui, &mut self.editors[active]);
            if let Some(action) = action {
                self.editor_action(action, ui.ctx());
            }
        }
    }
}
