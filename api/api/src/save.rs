use std::path::PathBuf;

use serde::{Serialize, de::DeserializeOwned};

use crate::{ErrorScreen, ScreenAction};

#[macro_export]
macro_rules! save_name {
    ($name:literal) => {
        const { $crate::SaveName::new(env!("CARGO_CRATE_NAME"), $name) }
    };
}

#[derive(Clone, Copy)]
pub struct SaveName {
    module: &'static str,
    name: &'static str,
}

impl SaveName {
    #[doc(hidden)]
    pub const fn new(module: &'static str, name: &'static str) -> Self {
        Self { module, name }
    }
}

fn save_path(name: SaveName) -> PathBuf {
    let filename = format!(
        "{}::{}.toml",
        name.module,
        name.name.trim_end_matches(".toml")
    );

    std::env::current_exe()
        .expect("Failed to get current executable path")
        .parent()
        .expect("Failed to get executable directory")
        .join(filename)
}

pub fn save<State, T: Serialize>(name: SaveName, save: &T) -> Result<(), ScreenAction<State>> {
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

pub fn load<State, T: Default + DeserializeOwned>(
    name: SaveName,
) -> Result<T, ScreenAction<State>> {
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

    toml::from_str(&content).map_err(|source| show_save_error(SaveError::Parse { path, source }))
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
    ScreenAction::Push(Box::new(ErrorScreen::new(error)))
}
