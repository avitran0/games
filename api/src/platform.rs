use std::{ffi::c_void, rc::Rc};

use glow::HasContext;
use sdl3::{
    GamepadSubsystem, VideoSubsystem,
    event::EventPollIterator,
    gamepad::Gamepad,
    video::{GLContext, GLProfile, SwapInterval, Window},
};
use thiserror::Error;
use utils::Level;

use crate::{HEIGHT, WIDTH, debug, info, warn};

pub struct Platform {
    gl: Rc<glow::Context>,
    _sdl_gl: GLContext,
    window: Window,
    _video: VideoSubsystem,
    event_pump: sdl3::EventPump,
    _gamepad: GamepadSubsystem,
    _gamepads: Vec<Gamepad>,
    _sdl: sdl3::Sdl,
}

impl Platform {
    pub fn load(title: &str) -> Result<Self, PlatformError> {
        let sdl = sdl3::init()?;
        let event_pump = sdl.event_pump()?;
        let _gamepad = sdl.gamepad()?;
        let _gamepads: Vec<_> = _gamepad
            .gamepads()?
            .into_iter()
            .filter_map(|id| match _gamepad.open(id) {
                Ok(gamepad) => Some(gamepad),
                Err(err) => {
                    warn!("failed to open gamepad {id:?}: {err}");
                    None
                }
            })
            .collect();
        info!("detected {} gamepads", _gamepads.len());
        let video = sdl.video()?;

        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(GLProfile::GLES);
        gl_attr.set_context_major_version(3);
        gl_attr.set_context_minor_version(0);

        let window = video
            .window(title, WIDTH * 2, HEIGHT * 2)
            .high_pixel_density()
            .opengl()
            .resizable()
            .build()?;

        let sdl_gl = window.gl_create_context()?;
        window.gl_make_current(&sdl_gl)?;
        video.gl_set_swap_interval(SwapInterval::VSync)?;

        let mut gl = unsafe {
            glow::Context::from_loader_function(|func| {
                video
                    .gl_get_proc_address(func)
                    .map(|ptr| ptr as *const c_void)
                    .unwrap_or(std::ptr::null())
            })
        };
        if gl.supported_extensions().contains("GL_KHR_debug") {
            unsafe {
                gl.enable(glow::DEBUG_OUTPUT);
                gl.debug_message_control(
                    glow::DONT_CARE,
                    glow::DONT_CARE,
                    glow::DEBUG_SEVERITY_NOTIFICATION,
                    &[],
                    false,
                );
                gl.debug_message_callback(gl_debug);
            }
            debug!("initialized OpenGL debug callback");
        }
        let gl = Rc::new(gl);

        Ok(Self {
            _sdl: sdl,
            event_pump,
            _gamepad,
            _gamepads,
            _video: video,
            window,
            _sdl_gl: sdl_gl,
            gl,
        })
    }

    pub(crate) fn events(&mut self) -> EventPollIterator<'_> {
        self.event_pump.poll_iter()
    }

    pub(crate) fn open_gamepad(&mut self, id: sdl3::joystick::JoystickId) {
        match self._gamepad.open(id) {
            Ok(gamepad) => self._gamepads.push(gamepad),
            Err(err) => warn!("failed to open gamepad {id:?}: {err}"),
        }
    }

    pub(crate) fn close_gamepad(&mut self, id: sdl3::joystick::JoystickId) {
        self._gamepads
            .retain(|gamepad| gamepad.id().ok() != Some(id));
    }

    pub(crate) fn swap_window(&self) {
        self.window.gl_swap_window();
    }

    pub(crate) fn window_size(&self) -> (u32, u32) {
        self.window.size_in_pixels()
    }

    pub(crate) fn gl_rc(&self) -> Rc<glow::Context> {
        self.gl.clone()
    }
}

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("SDL3 error: {0}")]
    Sdl(#[from] sdl3::Error),
    #[error("Cannot create window: {0}")]
    Window(#[from] sdl3::video::WindowBuildError),
    #[error("OpenGL error: {0}")]
    OpenGL(String),
    #[error("Cannot initialize assets: {0}")]
    Assets(String),
}

fn gl_debug(source: u32, _kind: u32, _id: u32, severity: u32, message: &str) {
    let level = match severity {
        glow::DEBUG_SEVERITY_LOW => Level::Info,
        glow::DEBUG_SEVERITY_MEDIUM => Level::Warn,
        glow::DEBUG_SEVERITY_HIGH => Level::Error,
        _ => return,
    };

    let source = match source {
        glow::DEBUG_SOURCE_API => "API",
        glow::DEBUG_SOURCE_WINDOW_SYSTEM => "Window System",
        glow::DEBUG_SOURCE_SHADER_COMPILER => "Shader Compiler",
        glow::DEBUG_SOURCE_THIRD_PARTY => "Third Party",
        glow::DEBUG_SOURCE_APPLICATION => "Application",
        _ => "Unknown",
    };

    utils::log!(level, "[GL/{source}] {message}");
}
