use api::{
    Button,
    glam::{IVec2, UVec2},
};

use crate::{difficulty::Difficulty, state::State};

pub struct Cursor {
    position: UVec2,
    difficulty: Difficulty,
    hold: Option<(Button, usize)>,
}

impl Cursor {
    const INITIAL_DELAY: usize = 40;
    const REPEAT_INTERVAL: usize = 20;
    const DIRECTIONS: [Button; 4] = [Button::Left, Button::Right, Button::Up, Button::Down];

    pub fn new(difficulty: Difficulty) -> Self {
        let position = difficulty.size() / 2;

        Self {
            difficulty,
            position,
            hold: None,
        }
    }

    pub fn pos(&self) -> UVec2 {
        self.position
    }

    pub fn tick(&mut self, ctx: &mut api::ScreenContext<State>) {
        let newly_pressed = Self::DIRECTIONS
            .into_iter()
            .find(|&button| ctx.input.just_pressed(button));

        if let Some(button) = newly_pressed {
            self.hold = Some((button, 0));
            self.move_cursor(button);
            return;
        }

        if let Some((button, elapsed)) = self.hold
            && ctx.input.is_pressed(button)
        {
            let elapsed = elapsed + 1;
            if elapsed >= Self::INITIAL_DELAY
                && (elapsed - Self::INITIAL_DELAY).is_multiple_of(Self::REPEAT_INTERVAL)
            {
                self.move_cursor(button);
            }
            self.hold = Some((button, elapsed));
            return;
        }

        if let Some(button) = Self::DIRECTIONS
            .into_iter()
            .find(|&button| ctx.input.is_pressed(button))
        {
            self.hold = Some((button, 0));
        } else {
            self.hold = None;
        }
    }

    fn move_cursor(&mut self, button: Button) {
        let size = self.difficulty.size();
        match button {
            Button::Left => self.position.x = self.position.x.saturating_sub(1),
            Button::Right => self.position.x = self.position.x.saturating_add(1).min(size.x - 1),
            Button::Up => self.position.y = self.position.y.saturating_sub(1),
            Button::Down => self.position.y = self.position.y.saturating_add(1).min(size.y - 1),
            _ => {}
        }
    }
}
