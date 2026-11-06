use std::{path::PathBuf, sync::Arc};

use egui::{Event as EguiEvent, Key, Modifiers};
use egui_glow::PainterError;
use sdl3::{
    event::Event,
    keyboard::{Keycode, Mod},
};

pub struct Egui {
    ctx: egui::Context,
    painter: egui_glow::Pointer,
    input: egui::RawInput,
    modifiers: egui::Modifiers,
}

impl Egui {
    pub fn new(gl: Arc<glow::Context>) -> Result<Self, PainterError> {
        let ctx = egui::Context::default();
        let painter = egui_glow::Painter::new(gl, "", None, false)?;
        let input = egui::RawInput::default();
        let modifiers = Modifiers::default();

        Ok(Self {
            ctx,
            painter,
            input,
            modifiers,
        })
    }

    pub(crate) fn handle_event(&mut self, event: &Event) {
        let event = self.convert_event(event);
    }

    fn convert_event(&mut self, event: &Event) {
        match event {
            Event::AppDidEnterBackground { .. } => Some(EguiEvent::WindowFocused(false)),
            Event::AppDidEnterForeground { .. } => Some(EguiEvent::WindowFocused(true)),
            Event::DropFile {
                timestamp,
                window_id,
                filename,
            } => {
                let file = Arc::new(PathDroppedFile {
                    path: PathBuf::from(filename),
                });
                self.input.dropped_files.push(file);
            }
            Event::KeyDown {
                keycode: Some(keycode),
                keymod,
                repeat,
                ..
            } => self.handle_key_event(*keycode, *keymod, true, *repeat),
            Event::KeyUp {
                keycode: Some(keycode),
                keymod,
                repeat,
                ..
            } => self.handle_key_event(*keycode, *keymod, false, *repeat),
        }
    }

    fn add_event(&mut self, event: EguiEvent) {}

    fn handle_key_event(
        &mut self,
        keycode: Keycode,
        keymod: Mod,
        pressed: bool,
        repeat: bool,
    ) -> Response {
        let key = key_to_egui(keycode)?;

        let modifiers = modifiers_to_egui(keymod);
        if modifiers != self.modifiers {
            self.modifiers = modifiers;
            self.add_event(EguiEvent::ModifiersChanged(modifiers));
        }

        self.add_event(EguiEvent::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat,
            modifiers: self.modifiers,
        });

        // when pressing tab, egui focuses the first focusable element
        let consumed = self.ctx.egui_wants_keyboard_input() || key == Key::Tab;
        Response {
            repaint: true,
            consumed,
        }
    }
}

struct Response {
    repaint: bool,
    consumed: bool,
}

struct PathDroppedFile {
    path: PathBuf,
}

impl egui::DroppedFile for PathDroppedFile {
    fn path(&self) -> &std::path::Path {
        &self.path
    }

    fn bytes(&self) -> Result<Vec<u8>, String> {
        std::fs::read(&self.path).map_err(|err| err.to_string())
    }
}

fn key_to_egui(key: Keycode) -> Key {
    match Keycode {}
}

fn modifiers_to_egui(modifiers: Mod) -> Modifiers {
    Modifiers {
        alt: modifiers.intersects(Mod::LALTMOD | Mod::RALTMOD),
        ctrl: modifiers.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD),
        shift: modifiers.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD),
        mac_cmd: false,
        command: (),
    }
}
