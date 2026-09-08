/// Identifies a static sprite. This handle has no animation controls.
#[derive(Clone, Copy)]
pub struct Sprite {
    pub(crate) id: SpriteId,
}

impl Sprite {
    pub(crate) fn new(id: SpriteId) -> Self {
        Self { id }
    }
}

/// Identifies a tilemap loaded into `Assets`.
#[derive(Clone, Copy)]
pub struct Tilemap {
    pub(crate) id: TilemapId,
}

impl Tilemap {
    pub(crate) fn new(id: TilemapId) -> Self {
        Self { id }
    }
}

/// Identifies an animated sprite. Each handle has separate playback state.
#[derive(Clone)]
pub struct AnimatedSprite {
    pub(crate) id: SpriteId,
    tick: u16,
    divisor: u16,
    animation: Option<String>,
}

impl AnimatedSprite {
    pub(crate) fn new(id: SpriteId) -> Self {
        Self {
            id,
            tick: 0,
            divisor: 1,
            animation: None,
        }
    }

    pub fn set_divisor(&mut self, divisor: u16) {
        self.divisor = divisor.max(1);
    }

    pub fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    pub fn set_animation(&mut self, name: impl Into<String>) {
        self.animation = Some(name.into());
        self.tick = 0;
    }

    pub fn clear_animation(&mut self) {
        self.animation = None;
        self.tick = 0;
    }

    pub(crate) fn animation(&self) -> Option<&str> {
        self.animation.as_deref()
    }

    pub(crate) fn animation_tick(&self) -> u16 {
        self.tick
    }

    pub(crate) fn animation_divisor(&self) -> u16 {
        self.divisor
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SpriteId(pub(crate) u16);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct TilemapId(pub(crate) u16);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Font(pub(crate) u16);
