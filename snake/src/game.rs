use api::{
    Anchor, Button,
    glam::{IVec2, ivec2},
};

use crate::{game_over::GameOverScreen, pause::PauseScreen, state::State};

const GRID_WIDTH: i32 = (api::WIDTH / 8) as i32;
const GRID_HEIGHT: i32 = (api::HEIGHT / 8) as i32;

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

pub struct GameScreen {
    sprite: api::AnimatedSprite,
    rng: api::Rng,
    snake: Vec<IVec2>,
    food: IVec2,
    direction: Direction,
    queued_direction: Direction,
}

impl GameScreen {
    pub fn new(ctx: &mut api::ScreenContext<State>) -> Result<Self, api::ScreenAction<State>> {
        let sprite = ctx
            .assets
            .load_animated_sprite(include_bytes!("assets/snake.pxa"))?;
        let start = ivec2(GRID_WIDTH / 2, GRID_HEIGHT / 2);

        Ok(Self {
            sprite,
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
                // Ends the game.
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
        for (index, &segment) in self.snake.iter().enumerate() {
            let (animation, rotation) = if index == 0 {
                ("Head", self.direction.rotation())
            } else if index + 1 == self.snake.len() {
                let direction = Direction::between(segment, self.snake[index - 1]);
                ("Tail", direction.rotation())
            } else {
                let from = Direction::between(self.snake[index], self.snake[index - 1]);
                let to = Direction::between(self.snake[index + 1], self.snake[index]);
                if from == to {
                    if from.vector().x != 0 {
                        ("BodyStraight", 90.0)
                    } else {
                        ("BodyStraight", 0.0)
                    }
                } else {
                    ("BodyAngled", corner_rotation(from, to))
                }
            };

            self.sprite.set_animation(animation);
            frame.animated_sprite_rotate(&self.sprite, segment * 8, rotation, Anchor::Center);
        }

        self.sprite.set_animation("Food");
        frame.animated_sprite(&self.sprite, self.food * 8);
    }
}

fn corner_rotation(from: Direction, to: Direction) -> f32 {
    match (from, to) {
        // Checks for a right turn.
        (Direction::Right, Direction::Up) => 90.0,
        (Direction::Down, Direction::Right) => 180.0,
        (Direction::Left, Direction::Down) => 270.0,
        (Direction::Up, Direction::Left) => 0.0,
        // Checks for a left turn.
        (Direction::Up, Direction::Right) => 270.0,
        (Direction::Right, Direction::Down) => 0.0,
        (Direction::Down, Direction::Left) => 90.0,
        (Direction::Left, Direction::Up) => 180.0,
        _ => 0.0,
    }
}
