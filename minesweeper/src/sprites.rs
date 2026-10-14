use crate::{difficulty::Difficulty, state::State};

pub struct Sprites {
    normal: SpriteSet,
    large: SpriteSet,
}

impl Sprites {
    pub fn load(ctx: &mut api::ScreenContext<State>) -> Result<Self, api::ScreenAction<State>> {
        let normal = SpriteSet::normal(ctx)?;
        let large = SpriteSet::large(ctx)?;

        Ok(Self { normal, large })
    }

    pub fn set(&self, difficulty: Difficulty) -> &SpriteSet {
        match difficulty {
            Difficulty::Easy => &self.large,
            Difficulty::Normal => &self.normal,
        }
    }
}

pub struct SpriteSet {
    pub cursor: api::Sprite,
    pub bomb: api::Sprite,
    pub flag: api::Sprite,
    pub covered: api::Sprite,
    pub uncovered: api::Sprite,
}

macro_rules! define_spriteset {
    ($($variant:ident => $dir:literal),* $(,)?) => {
        impl SpriteSet {
            $(pub fn $variant(
                ctx: &mut api::ScreenContext<State>,
            ) -> Result<Self, api::ScreenAction<State>> {
                let cursor =
                    ctx.assets
                    .load_sprite(include_bytes!(concat!("assets/", $dir, "/cursor.pxs")))?;
                let bomb = ctx
                    .assets
                    .load_sprite(include_bytes!(concat!("assets/", $dir, "/bomb.pxs")))?;
                let flag = ctx
                    .assets
                    .load_sprite(include_bytes!(concat!("assets/", $dir, "/flag.pxs")))?;
                let covered = ctx
                    .assets
                    .load_sprite(include_bytes!(concat!("assets/", $dir, "/covered.pxs")))?;
                let uncovered = ctx
                    .assets
                    .load_sprite(include_bytes!(concat!("assets/", $dir, "/uncovered.pxs")))?;

                Ok(Self { cursor, bomb, flag, covered, uncovered })
            })*
        }
    };
}

define_spriteset!(normal => "normal", large => "large");
