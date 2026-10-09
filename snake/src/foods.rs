use api::Rng;

use crate::state::State;

pub struct Foods {
    rng: Rng,
    foods: Vec<api::Sprite>,
}

impl Foods {
    pub fn load(ctx: &mut api::ScreenContext<State>) -> Result<Self, api::ScreenAction<State>> {
        let rng = Rng::new();
        let foods = FOODS
            .iter()
            .map(|file| ctx.assets.load_sprite(file))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { rng, foods })
    }

    pub fn random(&mut self) -> &api::Sprite {
        let index = self.rng.get_usize_range(0..self.len());
        &self.foods[index]
    }

    pub fn len(&self) -> usize {
        self.foods.len()
    }
}

macro_rules! include_foods {
    ($($dir:literal),+ $(,)?) => {
        const FOODS: &[&[u8]] = &[
            $(
                include_bytes!(concat!("assets/foods/", $dir, ".pxs")),
            )+
        ];
    };
}

include_foods!("food");
