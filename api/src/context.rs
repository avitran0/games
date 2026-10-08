use sdl3::{event::Event, gamepad::Button as GamepadButton, keyboard::Scancode};

use crate::{
    Assets, Frame, Input,
    input::Button,
    platform::{Platform, PlatformError},
    render::Renderer,
    screen::{Screen, ScreenAction, ScreenContext},
};

pub(crate) struct Context<State> {
    frame: Frame,
    input: Input,
    assets: Assets,
    state: State,
    renderer: Renderer,
    should_quit: bool,
    platform: Platform,
    screens: Vec<Box<dyn Screen<State>>>,
    tick: usize,
}

impl<State: 'static> Context<State> {
    pub(crate) fn load(title: &str, state: State) -> Result<Self, PlatformError> {
        let platform = Platform::load(title)?;
        let assets = Assets::new(platform.gl_rc()).map_err(PlatformError::Assets)?;
        let renderer = Renderer::new(platform.gl_rc()).map_err(PlatformError::OpenGL)?;

        Ok(Self {
            frame: Frame::default(),
            input: Input::default(),
            assets,
            state,
            renderer,
            should_quit: false,
            platform,
            screens: Vec::new(),
            tick: 0,
        })
    }

    fn poll_events(&mut self) {
        self.input.new_frame();
        let events: Vec<_> = self.platform.events().collect();
        for event in events {
            match event {
                Event::KeyDown {
                    scancode, repeat, ..
                } => {
                    let Some(key) = scancode else {
                        continue;
                    };
                    if repeat {
                        continue;
                    }
                    if let Some(button) = scancode_to_button(key) {
                        self.input.set_keyboard(button, true);
                    }
                }
                Event::KeyUp { scancode, .. } => {
                    let Some(key) = scancode else {
                        continue;
                    };
                    if let Some(button) = scancode_to_button(key) {
                        self.input.set_keyboard(button, false);
                    }
                }
                Event::GamepadAdded { which, .. } => {
                    self.platform.open_gamepad(which);
                }
                Event::GamepadRemoved { which, .. } => {
                    self.platform.close_gamepad(which);
                    self.input.clear_gamepad();
                }
                Event::GamepadButtonDown { button, .. } => {
                    if let Some(button) = gamepad_button_to_button(button) {
                        self.input.set_gamepad(button, true);
                    }
                }
                Event::GamepadButtonUp { button, .. } => {
                    if let Some(button) = gamepad_button_to_button(button) {
                        self.input.set_gamepad(button, false);
                    }
                }
                Event::Quit { .. } => self.should_quit = true,
                _ => {}
            }
        }
    }

    pub(crate) fn setup<F>(&mut self, setup: F)
    where
        F: FnOnce(
            &mut ScreenContext<'_, State>,
        ) -> Result<Box<dyn Screen<State>>, ScreenAction<State>>,
    {
        let mut screen_ctx = ScreenContext {
            input: &self.input,
            assets: &mut self.assets,
            state: &mut self.state,
            tick: self.tick,
        };
        match setup(&mut screen_ctx) {
            Ok(screen) => self.push_screen(screen),
            Err(action) => self.handle_action(action),
        }
    }

    pub(crate) fn begin_frame(&mut self) {
        self.poll_events();
        self.frame.clear();
        self.tick += 1;
    }

    pub(crate) fn end_frame(&self) {
        self.renderer.begin_frame();
        for cmd in self.frame.cmds() {
            self.renderer.draw(&self.assets, cmd);
        }
        self.renderer.end_frame(self.platform.window_size());
        self.platform.swap_window();
    }

    pub(crate) fn push_screen<S: Screen<State>>(&mut self, screen: S) {
        self.screens.push(Box::new(screen));
    }

    pub(crate) fn update_screens(&mut self) {
        let Some(screen) = self.screens.last_mut() else {
            return;
        };
        let mut screen_ctx = ScreenContext {
            input: &self.input,
            assets: &mut self.assets,
            state: &mut self.state,
            tick: self.tick,
        };
        let action = screen.update(&mut screen_ctx);
        self.handle_action(action);
    }

    fn handle_action(&mut self, action: ScreenAction<State>) {
        match action {
            ScreenAction::None => {}
            ScreenAction::Push(screen) => self.screens.push(screen),
            ScreenAction::ClearAndPush(screen) => {
                self.screens.clear();
                self.screens.push(screen);
            }
            ScreenAction::Pop => {
                self.screens.pop();
            }
            ScreenAction::Replace(screen) => {
                self.screens.pop();
                self.screens.push(screen);
            }
            ScreenAction::Quit => self.should_quit = true,
        }
    }

    pub(crate) fn draw_screens(&mut self) {
        let start = match self.screens.iter().rposition(|screen| screen.is_modal()) {
            Some(index) => index.saturating_sub(1),
            None => self.screens.len().saturating_sub(1),
        };

        for screen in &mut self.screens[start..] {
            screen.draw(&self.state, &mut self.frame);
        }
    }

    pub(crate) fn should_quit(&self) -> bool {
        self.should_quit
    }
}

fn gamepad_button_to_button(button: GamepadButton) -> Option<Button> {
    Some(match button {
        GamepadButton::DPadUp => Button::Up,
        GamepadButton::DPadDown => Button::Down,
        GamepadButton::DPadLeft => Button::Left,
        GamepadButton::DPadRight => Button::Right,
        // RG SP face-button labels: A right, B bottom, X top, Y left.
        GamepadButton::South => Button::B,
        GamepadButton::East => Button::A,
        GamepadButton::West => Button::Y,
        GamepadButton::North => Button::X,
        GamepadButton::LeftShoulder => Button::L,
        GamepadButton::RightShoulder => Button::R,
        GamepadButton::Start => Button::Start,
        GamepadButton::Back => Button::Select,
        _ => return None,
    })
}

fn scancode_to_button(key: Scancode) -> Option<Button> {
    Some(match key {
        Scancode::Up => Button::Up,
        Scancode::Down => Button::Down,
        Scancode::Left => Button::Left,
        Scancode::Right => Button::Right,
        Scancode::A => Button::A,
        Scancode::B => Button::B,
        Scancode::X => Button::X,
        Scancode::Y => Button::Y,
        Scancode::L => Button::L,
        Scancode::R => Button::R,
        Scancode::Return => Button::Start,
        Scancode::Backspace => Button::Select,
        _ => return None,
    })
}
