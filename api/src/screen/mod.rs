use serde::{Serialize, de::DeserializeOwned};

use crate::{Assets, Frame, Input, error, warn};

pub(crate) mod error;

pub struct ScreenContext<'a, State> {
    pub input: &'a Input,
    pub assets: &'a mut Assets,
    pub state: &'a mut State,
    pub tick: usize,
}

impl<'a, State> ScreenContext<'a, State> {
    pub fn save<T: Serialize>(name: &str, save: T) {
        let content = match toml::to_string(&save) {
            Ok(content) => content,
            Err(err) => {
                error!("Cannot serialize save '{name}': {err}");
                return;
            }
        };

        if let Err(err) = std::fs::write(name, content) {
            error!("Cannot write save '{name}': {err}");
        }
    }

    pub fn load<T: DeserializeOwned>(name: &str) -> Option<T> {
        let content = match std::fs::read_to_string(name) {
            Ok(content) => content,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return None,
            Err(err) => {
                warn!("Cannot read save '{name}': {err}");
                return None;
            }
        };

        match toml::from_str(&content) {
            Ok(save) => Some(save),
            Err(err) => {
                warn!("Cannot parse save '{name}': {err}");
                None
            }
        }
    }
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
