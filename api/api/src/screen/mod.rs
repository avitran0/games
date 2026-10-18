use crate::{Assets, Frame, Input, Resolution};

pub(crate) mod error;

pub struct ScreenContext<'a, State> {
    pub input: &'a Input,
    pub assets: &'a mut Assets,
    pub state: &'a mut State,
    pub tick: usize,
    pub resolution: Resolution,
}

impl<'a, State> ScreenContext<'a, State> {
    pub fn width(&self) -> u32 {
        self.resolution.width
    }

    pub fn height(&self) -> u32 {
        self.resolution.height
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
