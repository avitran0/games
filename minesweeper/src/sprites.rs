use crate::state::State;

#[derive(Clone, Copy)]
pub struct Sprites {
    pub cursor: api::Sprite,
    pub bomb: api::Sprite,
    pub flag: api::Sprite,
    pub covered: api::Sprite,
    pub uncovered: api::Sprite,
}

impl Sprites {
    pub fn load(ctx: &mut api::ScreenContext<State>) -> Result<Self, api::ScreenAction<State>> {
        let cursor = ctx
            .assets
            .load_sprite(include_bytes!("assets/cursor.pxs"))?;
        let bomb = ctx.assets.load_sprite(include_bytes!("assets/bomb.pxs"))?;
        let flag = ctx.assets.load_sprite(include_bytes!("assets/flag.pxs"))?;
        let covered = ctx
            .assets
            .load_sprite(include_bytes!("assets/covered.pxs"))?;
        let uncovered = ctx
            .assets
            .load_sprite(include_bytes!("assets/uncovered.pxs"))?;

        Ok(Self {
            cursor,
            bomb,
            flag,
            covered,
            uncovered,
        })
    }
}
