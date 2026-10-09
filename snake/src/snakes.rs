use crate::state::State;

pub struct Snakes {
    snakes: Vec<Snake>,
}

impl Snakes {
    pub fn load(ctx: &mut api::ScreenContext<State>) -> Result<Self, api::ScreenAction<State>> {
        let snakes = SNAKES
            .iter()
            .map(|files| Snake::load(ctx, files))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { snakes })
    }

    pub fn get(&self, index: usize) -> &Snake {
        &self.snakes[index]
    }

    pub fn len(&self) -> usize {
        self.snakes.len()
    }
}

pub struct Snake {
    pub head: api::Sprite,
    pub head_only: api::Sprite,
    pub body: api::Sprite,
    pub body_angled: api::Sprite,
    pub tail: api::Sprite,
}

struct SnakeFiles {
    head: &'static [u8],
    head_only: &'static [u8],
    body: &'static [u8],
    body_angled: &'static [u8],
    tail: &'static [u8],
}

impl Snake {
    fn load(
        ctx: &mut api::ScreenContext<State>,
        files: &SnakeFiles,
    ) -> Result<Self, api::ScreenAction<State>> {
        Ok(Self {
            head: ctx.assets.load_sprite(files.head)?,
            head_only: ctx.assets.load_sprite(files.head_only)?,
            body: ctx.assets.load_sprite(files.body)?,
            body_angled: ctx.assets.load_sprite(files.body_angled)?,
            tail: ctx.assets.load_sprite(files.tail)?,
        })
    }
}

macro_rules! include_snakes {
    ($($dir:literal),+ $(,)?) => {
        const SNAKES: &[SnakeFiles] = &[
            $(
                SnakeFiles {
                    head: include_bytes!(concat!("assets/snakes/", $dir, "/head.pxs")),
                    head_only: include_bytes!(concat!("assets/snakes/", $dir, "/head_only.pxs")),
                    body: include_bytes!(concat!("assets/snakes/", $dir, "/body.pxs")),
                    body_angled: include_bytes!(concat!("assets/snakes/", $dir, "/body_angled.pxs")),
                    tail: include_bytes!(concat!("assets/snakes/", $dir, "/tail.pxs")),
                },
            )+
        ];
    };
}

include_snakes!("default", "flower", "rainbow");
