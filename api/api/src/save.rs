use std::path::PathBuf;

use serde::{Serialize, de::DeserializeOwned};

use crate::{AppState, ErrorScreen, ScreenAction};

#[derive(Clone, Copy)]
pub struct SaveName {
    name: &'static str,
}

pub const fn save_name(name: &'static str) -> SaveName {
    SaveName { name }
}

#[cfg(target_os = "windows")]
pub(crate) fn game_config_dir(module: &str) -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .expect("LOCALAPPDATA is not set")
        .join("games")
        .join(module)
}

#[cfg(target_os = "linux")]
pub(crate) fn game_config_dir(module: &str) -> PathBuf {
    let config_home = config_home(
        std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
        std::env::var_os("HOME").map(PathBuf::from),
    )
    .expect("failed to locate config directory: set XDG_CONFIG_HOME or HOME");
    config_home.join("games").join(module)
}

#[cfg(target_os = "linux")]
fn config_home(xdg_config_home: Option<PathBuf>, home: Option<PathBuf>) -> Option<PathBuf> {
    xdg_config_home
        .filter(|path| path.is_absolute())
        .or_else(|| {
            home.filter(|path| path.is_absolute())
                .map(|path| path.join(".config"))
        })
}

fn save_path<State: AppState>(name: SaveName) -> PathBuf {
    game_config_dir(State::app_id()).join(format!("{}.toml", name.name.trim_end_matches(".toml")))
}

pub fn save<State: AppState, T: Serialize>(
    name: SaveName,
    save: &T,
) -> Result<(), ScreenAction<State>> {
    let path = save_path::<State>(name);
    let content = toml::to_string(&save).map_err(|source| {
        show_save_error(SaveError::Serialize {
            path: path.clone(),
            source,
        })
    })?;
    std::fs::create_dir_all(path.parent().expect("save path has a parent"))
        .and_then(|()| std::fs::write(&path, content))
        .map_err(|source| show_save_error(SaveError::Write { path, source }))?;
    Ok(())
}

pub fn load<State: AppState, T: Default + DeserializeOwned>(
    name: SaveName,
) -> Result<T, ScreenAction<State>> {
    let path = save_path::<State>(name);
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
