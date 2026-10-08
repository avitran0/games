use api::{
    Anchor, Button, Flip,
    glam::{IVec2, ivec2},
};

use crate::{game_over::GameOverScreen, pause::PauseScreen, state::State};

const TILE_SIZE: i32 = 16;
const GRID_WIDTH: i32 = (api::WIDTH / TILE_SIZE as u32) as i32;
const GRID_HEIGHT: i32 = (api::HEIGHT / TILE_SIZE as u32) as i32;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn vector(self) -> IVec2 {
        match self {
            Self::Up => -IVec2::Y,
            Self::Down => IVec2::Y,
            Self::Left => -IVec2::X,
            Self::Right => IVec2::X,
        }
    }

    fn rotation(self) -> f32 {
        match self {
            Self::Up => 0.0,
            Self::Right => 90.0,
            Self::Down => 180.0,
            Self::Left => 270.0,
        }
    }

    fn between(from: IVec2, to: IVec2) -> Self {
        let delta = to - from;
        let horizontal = delta.x.rem_euclid(GRID_WIDTH);
        let vertical = delta.y.rem_euclid(GRID_HEIGHT);

        if horizontal == 1 {
            Self::Right
        } else if horizontal == GRID_WIDTH - 1 {
            Self::Left
        } else if vertical == 1 {
            Self::Down
        } else {
            Self::Up
        }
    }
}

struct Sprites {
    head: api::Sprite,
    head_only: api::Sprite,
    body: api::Sprite,
    body_angled: api::Sprite,
    tail: api::Sprite,
    food: api::Sprite,
    background: api::Tilemap,
}

impl Sprites {
    fn load(ctx: &mut api::ScreenContext<State>) -> Result<Self, api::ScreenAction<State>> {
        let head = ctx
            .assets
            .load_sprite(include_bytes!("assets/head_16.pxs"))?;
        let head_only = ctx
            .assets
            .load_sprite(include_bytes!("assets/head_only_16.pxs"))?;
        let body = ctx
            .assets
            .load_sprite(include_bytes!("assets/body_16.pxs"))?;
        let body_angled = ctx
            .assets
            .load_sprite(include_bytes!("assets/body_angled_16.pxs"))?;
        let tail = ctx
            .assets
            .load_sprite(include_bytes!("assets/tail_16.pxs"))?;
        let food = ctx
            .assets
            .load_sprite(include_bytes!("assets/food_16.pxs"))?;
        let _ = ctx
            .assets
            .load_tileset(include_bytes!("assets/tileset_16.pxt"))?;
        let background = ctx
            .assets
            .load_tilemap(include_bytes!("assets/map_16.pxm"))?;

        Ok(Self {
            head,
            head_only,
            body,
            body_angled,
            tail,
            food,
            background,
        })
    }
}

pub struct GameScreen {
    sprites: Sprites,
    rng: api::Rng,
    snake: Vec<IVec2>,
    food: IVec2,
    direction: Direction,
    queued_direction: Direction,
}

impl GameScreen {
    pub fn new(ctx: &mut api::ScreenContext<State>) -> Result<Self, api::ScreenAction<State>> {
        let sprites = Sprites::load(ctx)?;
        let start = ivec2(GRID_WIDTH / 2, GRID_HEIGHT / 2);

        Ok(Self {
            sprites,
            rng: api::Rng::new(),
            snake: vec![start],
            food: ivec2(5, 5),
            direction: Direction::Right,
            queued_direction: Direction::Right,
        })
    }

    fn spawn_food(&mut self) {
        let free: Vec<IVec2> = (0..GRID_HEIGHT)
            .flat_map(|y| (0..GRID_WIDTH).map(move |x| ivec2(x, y)))
            .filter(|position| !self.snake.contains(position))
            .collect();

        if !free.is_empty() {
            let index = self.rng.get_usize_range(0..free.len());
            self.food = free[index];
        }
    }

    fn reset(&mut self) {
        let start = ivec2(GRID_WIDTH / 2, GRID_HEIGHT / 2);
        self.snake.clear();
        self.snake.push(start);
        self.direction = Direction::Right;
        self.queued_direction = Direction::Right;
        self.spawn_food();
    }
}

impl api::Screen<State> for GameScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        if ctx.input.just_pressed(Button::Up) && self.direction != Direction::Down {
            self.queued_direction = Direction::Up;
        }
        if ctx.input.just_pressed(Button::Down) && self.direction != Direction::Up {
            self.queued_direction = Direction::Down;
        }
        if ctx.input.just_pressed(Button::Left) && self.direction != Direction::Right {
            self.queued_direction = Direction::Left;
        }
        if ctx.input.just_pressed(Button::Right) && self.direction != Direction::Left {
            self.queued_direction = Direction::Right;
        }

        let move_interval = ctx.state.difficulty.move_interval();
        if ctx.tick.is_multiple_of(move_interval) {
            self.direction = self.queued_direction;
            let head = self.snake[0];
            let next = (head + self.direction.vector()).rem_euclid(ivec2(GRID_WIDTH, GRID_HEIGHT));
            let growing = next == self.food;

            let collision_len = if growing {
                self.snake.len()
            } else {
                self.snake.len().saturating_sub(1)
            };
            if self.snake[..collision_len].contains(&next) {
                self.reset();
                return api::ScreenAction::Replace(Box::new(GameOverScreen));
            } else {
                self.snake.insert(0, next);
                if growing {
                    ctx.state.score += 1;
                    self.spawn_food();
                } else {
                    self.snake.pop();
                }
            }
        }

        if ctx.input.just_pressed(Button::Start) {
            return api::ScreenAction::Push(Box::new(PauseScreen));
        }
        api::ScreenAction::None
    }

    fn draw(&mut self, _state: &State, frame: &mut api::Frame) {
        frame.tilemap(&self.sprites.background, ivec2(0, 0));

        for (index, &segment) in self.snake.iter().enumerate() {
            let (sprite, rotation, flip) = if self.snake.len() == 1 {
                (
                    &self.sprites.head_only,
                    self.direction.rotation(),
                    Flip::None,
                )
            } else if index == 0 {
                (&self.sprites.head, self.direction.rotation(), Flip::None)
            } else if index + 1 == self.snake.len() {
                let direction = Direction::between(segment, self.snake[index - 1]);
                (&self.sprites.tail, direction.rotation(), Flip::None)
            } else {
                let from = Direction::between(self.snake[index], self.snake[index - 1]);
                let to = Direction::between(self.snake[index + 1], self.snake[index]);
                if from == to {
                    (&self.sprites.body, from.rotation(), Flip::None)
                } else {
                    let (rotation, flip) = corner_rotation(from, to);
                    (&self.sprites.body_angled, rotation, flip)
                }
            };

            frame.sprite_rotate_flip(sprite, segment * TILE_SIZE, rotation, Anchor::Center, flip);
        }

        frame.sprite(&self.sprites.food, self.food * TILE_SIZE);
    }
}

fn corner_rotation(from: Direction, to: Direction) -> (f32, Flip) {
    match (from, to) {
        // right turn, no flip
        (Direction::Up, Direction::Left) => (0.0, Flip::None),
        (Direction::Right, Direction::Up) => (90.0, Flip::None),
        (Direction::Down, Direction::Right) => (180.0, Flip::None),
        (Direction::Left, Direction::Down) => (270.0, Flip::None),
        // left turn, including flip, otherwise the sprites don't connect properly
        (Direction::Up, Direction::Right) => (0.0, Flip::Horizontal),
        (Direction::Right, Direction::Down) => (90.0, Flip::Horizontal),
        (Direction::Down, Direction::Left) => (180.0, Flip::Horizontal),
        (Direction::Left, Direction::Up) => (270.0, Flip::Horizontal),
        _ => (0.0, Flip::None),
    }
}
