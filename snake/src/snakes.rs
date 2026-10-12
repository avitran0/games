use crate::state::State;

macro_rules! snake_files {
    ($dir:literal) => {
        SnakeFiles {
            head: include_bytes!(concat!("assets/snakes/", $dir, "/head.pxs")),
            head_only: include_bytes!(concat!("assets/snakes/", $dir, "/head_only.pxs")),
            body: include_bytes!(concat!("assets/snakes/", $dir, "/body.pxs")),
            body_angled: include_bytes!(concat!("assets/snakes/", $dir, "/body_angled.pxs")),
            tail: include_bytes!(concat!("assets/snakes/", $dir, "/tail.pxs")),
        }
    };
}

macro_rules! define_snakes {
    ($default:ident => $default_dir:literal $(, $variant:ident => $dir:literal)* $(,)?) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ui::EnumIter, api::serde::Serialize, api::serde::Deserialize)]
        #[serde(crate = "api::serde")]
        #[strum(crate = "ui")]
        pub enum Snake {
            #[default]
            $default,
            $($variant,)*
        }

        impl Snake {
            pub fn load(
                self,
                ctx: &mut api::ScreenContext<State>,
            ) -> Result<SnakeSprites, api::ScreenAction<State>> {
                match self {
                    Self::$default => SnakeSprites::load(ctx, snake_files!($default_dir)),
                    $(Self::$variant => SnakeSprites::load(ctx, snake_files!($dir)),)*
                }
            }

            pub fn load_all(
                ctx: &mut api::ScreenContext<State>,
            ) -> Result<Vec<SnakeSprites>, api::ScreenAction<State>> {
                use ui::IntoEnumIterator;

                Self::iter().map(|snake| snake.load(ctx)).collect()
            }

            pub fn index(self) -> usize {
                self as usize
            }
        }
    };
}

pub struct SnakeSprites {
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

impl SnakeSprites {
    fn load(
        ctx: &mut api::ScreenContext<State>,
        files: SnakeFiles,
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

define_snakes!(
    Default => "default",
    Flower => "flower",
    Rainbow => "rainbow",
    Galaxy => "galaxy",
    Dragon => "dragon",
    Rainbow2 => "rainbow2",
);
