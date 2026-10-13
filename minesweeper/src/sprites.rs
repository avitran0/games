use crate::state::State;

pub struct Sprites {
    bomb: api::Sprite,
    flag: api::Sprite,
}

impl Sprites {
    pub fn load(ctx: &mut api::ScreenContext<State>) -> Result<Self, api::ScreenAction<State>> {
        let bomb = ctx.assets.load_sprite(include_bytes!("assets/bomb.pxs"))?;
        let flag = ctx.assets.load_sprite(include_bytes!("assets/flag.pxs"))?;

        Ok(Self { bomb, flag })
    }
}
