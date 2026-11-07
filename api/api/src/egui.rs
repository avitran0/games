use std::{path::PathBuf, sync::Arc};

use egui::{Event as EguiEvent, Key, Modifiers, PointerButton, pos2};
use egui_glow::PainterError;
use sdl3::{
    event::{Event, WindowEvent},
    keyboard::{Keycode, Mod},
    mouse::MouseButton,
};

pub struct Egui {
    ctx: egui::Context,
    painter: egui_glow::Painter,
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
        let response = self.convert_event(event);
    }

    fn convert_event(&mut self, event: &Event) -> Response {
        match event {
            Event::DropFile {
                timestamp,
                window_id,
                filename,
            } => {
                let file = Arc::new(PathDroppedFile {
                    path: PathBuf::from(filename),
                });
                self.input.dropped_files.push(file);
                Response {
                    repaint: true,
                    consumed: false,
                }
            }
            Event::KeyDown {
                keycode: Some(keycode),
                keymod,
                repeat,
                ..
                // deliberately does not handle copy/cut/paste
            } => self.handle_key_event(*keycode, *keymod, true, *repeat),
            Event::KeyUp {
                keycode: Some(keycode),
                keymod,
                repeat,
                ..
            } => self.handle_key_event(*keycode, *keymod, false, *repeat),
            Event::MouseButtonDown { mouse_btn, x, y,.. } => self.handle_mouse_button_event(*mouse_btn, *x, *y, true),
            Event::MouseButtonUp { mouse_btn,  x, y ,..} => self.handle_mouse_button_event(*mouse_btn, *x, *y, false),
            Event::Window {  win_event,.. } => self.handle_window_event(win_event),
            _ =>

            Response::default()
        }
    }

    fn add_event(&mut self, event: EguiEvent) {}

    fn handle_mouse_button_event(
        &mut self,
        button: MouseButton,
        x: f32,
        y: f32,
        pressed: bool,
    ) -> Response {
        let button = mouse_btn_to_egui(button);
        let pos = pos2(x, y);
        self.add_event(EguiEvent::PointerButton {
            pos,
            button,
            pressed,
            modifiers: self.modifiers,
        });

        let consumed = self.ctx.egui_wants_pointer_input();
        Response {
            repaint: true,
            consumed,
        }
    }

    fn handle_key_event(
        &mut self,
        keycode: Keycode,
        keymod: Mod,
        pressed: bool,
        repeat: bool,
    ) -> Response {
        let Some(key) = key_to_egui(keycode) else {
            return Response::default();
        };

        let modifiers = modifiers_to_egui(keymod);
        if modifiers != self.modifiers {
            self.modifiers = modifiers;
            self.add_event(EguiEvent::ModifiersChanged(modifiers));
        }

        self.add_event(EguiEvent::Key {
            key,
            physical_key: None,
            pressed,
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

    fn handle_window_event(&mut self, win_event: &WindowEvent) -> Response {
        match win_event {
            WindowEvent::FocusGained => {
                self.input.focused = true;
                self.add_event(EguiEvent::WindowFocused(true));
                Response {
                    repaint: true,
                    consumed: false,
                }
            }
            WindowEvent::FocusLost => {
                self.input.focused = false;
                self.add_event(EguiEvent::WindowFocused(false));
                Response {
                    repaint: true,
                    consumed: false,
                }
            }
            WindowEvent::MouseLeave => {
                self.add_event(EguiEvent::PointerGone);
                Response {
                    repaint: true,
                    consumed: false,
                }
            }
            _ => Response {
                repaint: true,
                consumed: false,
            },
        }
    }
}

#[derive(Default)]
struct Response {
    repaint: bool,
    consumed: bool,
}

#[derive(Debug)]
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

fn mouse_btn_to_egui(btn: MouseButton) -> PointerButton {
    match btn {
        MouseButton::Left => PointerButton::Primary,
        MouseButton::Right => PointerButton::Secondary,
        MouseButton::Middle => PointerButton::Middle,
        MouseButton::X1 => PointerButton::Extra1,
        MouseButton::X2 => PointerButton::Extra2,
        MouseButton::Unknown => PointerButton::Primary,
    }
}

fn key_to_egui(key: Keycode) -> Option<Key> {
    Some(match key {
        Keycode::Left => Key::ArrowLeft,
        Keycode::Up => Key::ArrowUp,
        Keycode::Right => Key::ArrowRight,
        Keycode::Down => Key::ArrowDown,

        Keycode::Escape => Key::Escape,
        Keycode::Tab => Key::Tab,
        Keycode::Backspace => Key::Backspace,
        Keycode::Space => Key::Space,
        Keycode::Return => Key::Enter,

        Keycode::Insert => Key::Insert,
        Keycode::Home => Key::Home,
        Keycode::Delete => Key::Delete,
        Keycode::End => Key::End,
        Keycode::PageDown => Key::PageDown,
        Keycode::PageUp => Key::PageUp,

        Keycode::Kp0 | Keycode::_0 => Key::Num0,
        Keycode::Kp1 | Keycode::_1 => Key::Num1,
        Keycode::Kp2 | Keycode::_2 => Key::Num2,
        Keycode::Kp3 | Keycode::_3 => Key::Num3,
        Keycode::Kp4 | Keycode::_4 => Key::Num4,
        Keycode::Kp5 | Keycode::_5 => Key::Num5,
        Keycode::Kp6 | Keycode::_6 => Key::Num6,
        Keycode::Kp7 | Keycode::_7 => Key::Num7,
        Keycode::Kp8 | Keycode::_8 => Key::Num8,
        Keycode::Kp9 | Keycode::_9 => Key::Num9,

        Keycode::A => Key::A,
        Keycode::B => Key::B,
        Keycode::C => Key::C,
        Keycode::D => Key::D,
        Keycode::E => Key::E,
        Keycode::F => Key::F,
        Keycode::G => Key::G,
        Keycode::H => Key::H,
        Keycode::I => Key::I,
        Keycode::J => Key::J,
        Keycode::K => Key::K,
        Keycode::L => Key::L,
        Keycode::M => Key::M,
        Keycode::N => Key::N,
        Keycode::O => Key::O,
        Keycode::P => Key::P,
        Keycode::Q => Key::Q,
        Keycode::R => Key::R,
        Keycode::S => Key::S,
        Keycode::T => Key::T,
        Keycode::U => Key::U,
        Keycode::V => Key::V,
        Keycode::W => Key::W,
        Keycode::X => Key::X,
        Keycode::Y => Key::Y,
        Keycode::Z => Key::Z,

        Keycode::F1 => Key::F1,
        Keycode::F2 => Key::F2,
        Keycode::F3 => Key::F3,
        Keycode::F4 => Key::F4,
        Keycode::F5 => Key::F5,
        Keycode::F6 => Key::F6,
        Keycode::F7 => Key::F7,
        Keycode::F8 => Key::F8,
        Keycode::F9 => Key::F9,
        Keycode::F10 => Key::F10,
        Keycode::F11 => Key::F11,
        Keycode::F12 => Key::F12,

        Keycode::Minus => Key::Minus,
        Keycode::Equals => Key::Equals,
        Keycode::Semicolon => Key::Semicolon,
        Keycode::Comma => Key::Comma,
        Keycode::Period => Key::Period,
        Keycode::Slash => Key::Slash,
        Keycode::Backslash => Key::Backslash,

        _ => return None,
    })
}

fn modifiers_to_egui(modifiers: Mod) -> Modifiers {
    let ctrl = modifiers.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD);
    let gui = modifiers.intersects(Mod::LGUIMOD | Mod::RGUIMOD);

    Modifiers {
        alt: modifiers.intersects(Mod::LALTMOD | Mod::RALTMOD),
        ctrl,
        shift: modifiers.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD),
        mac_cmd: gui,
        command: ctrl || gui,
    }
}
