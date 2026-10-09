use std::path::PathBuf;

use serde::{Serialize, de::DeserializeOwned};

use crate::{Assets, Frame, Input};

fn save_path(name: &str) -> PathBuf {
    if name.ends_with(".toml") {
        PathBuf::from(name)
    } else {
        PathBuf::from(format!("{name}.toml"))
    }
}

pub(crate) mod error;

pub struct ScreenContext<'a, State> {
    pub input: &'a Input,
    pub assets: &'a mut Assets,
    pub state: &'a mut State,
    pub tick: usize,
}

impl<'a, State> ScreenContext<'a, State> {
    pub fn save<T: Serialize>(name: &str, save: T) -> Result<(), ScreenAction<State>> {
        let path = save_path(name);
        let content = toml::to_string(&save).map_err(|source| {
            show_save_error(SaveError::Serialize {
                path: path.clone(),
                source,
            })
        })?;
        std::fs::write(&path, content)
            .map_err(|source| show_save_error(SaveError::Write { path, source }))?;
        Ok(())
    }

    pub fn load<T: Default + DeserializeOwned>(name: &str) -> Result<T, ScreenAction<State>> {
        let path = save_path(name);
        let content = match std::fs::read_to_string(&path) {
            Ok(content) => content,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                return Ok(T::default());
            }
            Err(source) => {
                return Err(show_save_error(SaveError::Read { path, source }));
            }
        };

        toml::from_str(&content)
            .map_err(|source| show_save_error(SaveError::Parse { path, source }))
    }
}

#[derive(Debug, thiserror::Error)]
enum SaveError {
    #[error("failed to serialize save '{path}': {source}")]
    Serialize {
        path: PathBuf,
        #[source]
        source: toml::ser::Error,
    },
    #[error("failed to write save '{path}': {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to read save '{path}': {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse save '{path}': {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
}

fn show_save_error<State>(error: SaveError) -> ScreenAction<State> {
    ScreenAction::Push(Box::new(error::ErrorScreen::new(error)))
}

pub trait Screen<State>: 'static {
    fn update(&mut self, ctx: &mut ScreenContext<'_, State>) -> ScreenAction<State>;
    fn draw(&mut self, state: &State, frame: &mut Frame);
    fn is_modal(&self) -> bool {
        false
    }
}

impl<State: 'static> Screen<State> for Box<dyn Screen<State>> {
    fn update(&mut self, ctx: &mut ScreenContext<'_, State>) -> ScreenAction<State> {
        (**self).update(ctx)
    }

    fn draw(&mut self, state: &State, frame: &mut Frame) {
        (**self).draw(state, frame);
    }

    fn is_modal(&self) -> bool {
        (**self).is_modal()
    }
}

pub enum ScreenAction<State> {
    None,
    Push(Box<dyn Screen<State>>),
    ClearAndPush(Box<dyn Screen<State>>),
    Pop,
    Replace(Box<dyn Screen<State>>),
    Quit,
}
