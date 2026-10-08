use glam::Vec2;

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum Button {
    Up = 1 << 0,
    Down = 1 << 1,
    Left = 1 << 2,
    Right = 1 << 3,
    A = 1 << 4,
    B = 1 << 5,
    X = 1 << 6,
    Y = 1 << 7,
    L = 1 << 8,
    R = 1 << 9,
    Start = 1 << 10,
    Select = 1 << 11,
}

impl Button {
    fn bit(self) -> u16 {
        self as u16
    }
}

#[derive(Clone, Copy, Default)]
pub struct Input {
    previous: u16,
    keyboard: u16,
    gamepad: u16,
}

impl Input {
    pub(crate) fn new_frame(&mut self) {
        self.previous = self.current();
    }

    pub(crate) fn set_keyboard(&mut self, button: Button, down: bool) {
        Self::set_bit(&mut self.keyboard, button, down);
    }

    pub(crate) fn set_gamepad(&mut self, button: Button, down: bool) {
        Self::set_bit(&mut self.gamepad, button, down);
    }

    pub(crate) fn clear_gamepad(&mut self) {
        self.gamepad = 0;
    }

    fn set_bit(bits: &mut u16, button: Button, down: bool) {
        if down {
            *bits |= button.bit();
        } else {
            *bits &= !button.bit();
        }
    }

    fn current(&self) -> u16 {
        self.keyboard | self.gamepad
    }

    fn down(input: u16, button: Button) -> bool {
        input & button.bit() != 0
    }

    pub fn is_pressed(&self, button: Button) -> bool {
        Self::down(self.current(), button)
    }

    pub fn just_pressed(&self, button: Button) -> bool {
        Self::down(self.current(), button) && !Self::down(self.previous, button)
    }

    pub fn vector(&self) -> Vec2 {
        let mut out = Vec2::ZERO;
        if self.is_pressed(Button::Left) {
            out.x -= 1.0;
        }
        if self.is_pressed(Button::Right) {
            out.x += 1.0;
        }
        if self.is_pressed(Button::Up) {
            out.y -= 1.0;
        }
        if self.is_pressed(Button::Down) {
            out.y += 1.0;
        }
        out.normalize_or_zero()
    }
}
