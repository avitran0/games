use super::delayed_tween::DelayedTween;

pub(super) const REVEAL_ANIMATION_TICKS: usize = 20;
pub(super) const REVEAL_STAGGER_GROUP_SIZE: usize = 4;
pub(super) const REVEAL_STAGGER_TICKS: usize = 2;
pub(super) const CHORD_STAGGER_TICKS: usize = 2;
pub(super) const WIN_FLAG_ANIMATION_TICKS: usize = 20;
pub(super) const WIN_FLAG_STAGGER_GROUP_SIZE: usize = 4;
pub(super) const WIN_FLAG_STAGGER_TICKS: usize = 2;
pub(super) const BOMB_REVEAL_STAGGER_GROUP_SIZE: usize = 4;
pub(super) const BOMB_REVEAL_STAGGER_TICKS: usize = 2;

#[derive(Clone, Default)]
pub(super) struct TileAnimation {
    pub(super) reveal: Option<DelayedTween>,
    pub(super) flag: Option<FlagAnimation>,
}

#[derive(Clone)]
pub(super) struct FlagAnimation {
    pub(super) tween: DelayedTween,
}
