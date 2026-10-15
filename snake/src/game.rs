use api::{
    Anchor, Button, Flip, SpriteOptions,
    glam::{IVec2, ivec2},
};

use crate::{
    end_screen::{EndScreen, Outcome},
    foods::Foods,
    pause::PauseScreen,
    snakes::{Snake, SnakeSprites},
    state::State,
};

const TILE_SIZE: i32 = 16;
const GRID_WIDTH: i32 = (api::WIDTH / TILE_SIZE as u32) as i32;
const GRID_HEIGHT: i32 = (api::HEIGHT / TILE_SIZE as u32) as i32;
const FOOD_POINTS: u32 = 10;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn can_turn_to(self, next: Self) -> bool {
        !matches!(
            (self, next),
            (Self::Up, Self::Down)
                | (Self::Down, Self::Up)
                | (Self::Left, Self::Right)
                | (Self::Right, Self::Left)
        )
    }

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
    snakes: Vec<SnakeSprites>,
    foods: Foods,
    background: api::Tilemap,
}

impl Sprites {
    fn load(ctx: &mut api::ScreenContext<State>) -> Result<Self, api::ScreenAction<State>> {
        let snakes = Snake::load_all(ctx)?;
        let foods = Foods::load(ctx)?;
        let _ = ctx
            .assets
            .load_tileset(include_bytes!("assets/tileset_16.pxt"))?;
        let background = ctx
            .assets
            .load_tilemap(include_bytes!("assets/map_16.pxm"))?;

        Ok(Self {
            snakes,
            foods,
            background,
        })
    }
}

pub struct GameScreen {
    sprites: Sprites,
    rng: api::Rng,
    snake: Vec<IVec2>,
    food: Option<(api::Sprite, IVec2)>,
    direction: Direction,
    queued_direction: Direction,
}

impl GameScreen {
    pub fn new(ctx: &mut api::ScreenContext<State>) -> Result<Self, api::ScreenAction<State>> {
        let sprites = Sprites::load(ctx)?;
        let start = ivec2(GRID_WIDTH / 2, GRID_HEIGHT / 2);
        let mut game = Self {
            sprites,
            rng: api::Rng::new(),
            snake: initial_snake(start),
            food: None,
            direction: Direction::Right,
            queued_direction: Direction::Right,
        };
        game.spawn_food();
        Ok(game)
    }

    fn spawn_food(&mut self) -> bool {
        let free = free_positions(&self.snake);
        if free.is_empty() {
            self.food = None;
            return false;
        }

        let index = self.rng.get_usize_range(0..free.len());
        let sprite = *self.sprites.foods.random();
        self.food = Some((sprite, free[index]));
        true
    }
}

impl api::Screen<State> for GameScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> api::ScreenAction<State> {
        if ctx.input.just_pressed(Button::Up) && self.direction.can_turn_to(Direction::Up) {
            self.queued_direction = Direction::Up;
        }
        if ctx.input.just_pressed(Button::Down) && self.direction.can_turn_to(Direction::Down) {
            self.queued_direction = Direction::Down;
        }
        if ctx.input.just_pressed(Button::Left) && self.direction.can_turn_to(Direction::Left) {
            self.queued_direction = Direction::Left;
        }
        if ctx.input.just_pressed(Button::Right) && self.direction.can_turn_to(Direction::Right) {
            self.queued_direction = Direction::Right;
        }

        let move_interval = ctx.state.difficulty.move_interval();
        if ctx.tick.is_multiple_of(move_interval) {
            self.direction = self.queued_direction;
            let head = self.snake[0];
            let next = next_position(head, self.direction);
            let growing = self
                .food
                .as_ref()
                .is_some_and(|(_, position)| next == *position);

            if hits_snake(&self.snake, next, growing) {
                return api::ScreenAction::Replace(Box::new(EndScreen::new(Outcome::GameOver)));
            }

            self.snake.insert(0, next);
            if growing {
                ctx.state.score = score_after_food(ctx.state.score);
                if !self.spawn_food() {
                    return api::ScreenAction::Replace(Box::new(EndScreen::new(Outcome::Win)));
                }
            } else {
                self.snake.pop();
            }
        }

        if ctx.input.just_pressed(Button::Start) {
            return api::ScreenAction::Push(Box::new(PauseScreen::new()));
        }
        api::ScreenAction::None
    }

    fn draw(&mut self, state: &State, frame: &mut api::Frame) {
        frame.tilemap(
            &self.sprites.background,
            ivec2(0, 0),
            api::TilemapOptions::default(),
        );

        for (index, &segment) in self.snake.iter().enumerate() {
            let snake = &self.sprites.snakes[state.config.snake.index()];
            let (sprite, rotation, flip) = if index == 0 {
                (&snake.head, self.direction.rotation(), Flip::None)
            } else if index + 1 == self.snake.len() {
                let direction = Direction::between(segment, self.snake[index - 1]);
                (&snake.tail, direction.rotation(), Flip::None)
            } else {
                let from = Direction::between(self.snake[index], self.snake[index - 1]);
                let to = Direction::between(self.snake[index + 1], self.snake[index]);
                if from == to {
                    (&snake.body, from.rotation(), Flip::None)
                } else {
                    let (rotation, flip) = corner_rotation(from, to);
                    (&snake.body_angled, rotation, flip)
                }
            };

            frame.sprite(
                sprite,
                segment * TILE_SIZE,
                SpriteOptions::default()
                    .rotation(rotation)
                    .rotation_anchor(Anchor::Center)
                    .flip(flip),
            );
        }

        if let Some((sprite, position)) = &self.food {
            frame.sprite(sprite, *position * TILE_SIZE, SpriteOptions::default());
        }

        frame.text(
            format!("Score: {}", state.score),
            ivec2(4, 4),
            Anchor::TopLeft,
        );
    }
}

fn score_after_food(score: u32) -> u32 {
    score.saturating_add(FOOD_POINTS)
}

fn initial_snake(start: IVec2) -> Vec<IVec2> {
    vec![start, next_position(start, Direction::Left)]
}

fn next_position(position: IVec2, direction: Direction) -> IVec2 {
    (position + direction.vector()).rem_euclid(ivec2(GRID_WIDTH, GRID_HEIGHT))
}

fn hits_snake(snake: &[IVec2], position: IVec2, growing: bool) -> bool {
    let body_len = if growing {
        snake.len()
    } else {
        snake.len().saturating_sub(1)
    };
    snake[..body_len].contains(&position)
}

fn free_positions(snake: &[IVec2]) -> Vec<IVec2> {
    (0..GRID_HEIGHT)
        .flat_map(|y| (0..GRID_WIDTH).map(move |x| ivec2(x, y)))
        .filter(|position| !snake.contains(position))
        .collect()
}

fn corner_rotation(from: Direction, to: Direction) -> (f32, Flip) {
    match (from, to) {
        (Direction::Up, Direction::Left) => (0.0, Flip::None),
        (Direction::Right, Direction::Up) => (90.0, Flip::None),
        (Direction::Down, Direction::Right) => (180.0, Flip::None),
        (Direction::Left, Direction::Down) => (270.0, Flip::None),
        // flip the sprite so its ends connect.
        (Direction::Up, Direction::Right) => (0.0, Flip::Horizontal),
        (Direction::Right, Direction::Down) => (90.0, Flip::Horizontal),
        (Direction::Down, Direction::Left) => (180.0, Flip::Horizontal),
        (Direction::Left, Direction::Up) => (270.0, Flip::Horizontal),
        _ => (0.0, Flip::None),
    }
}

#[cfg(test)]
mod tests {
    use api::{Flip, glam::ivec2};

    use super::{
        Direction, FOOD_POINTS, GRID_HEIGHT, GRID_WIDTH, corner_rotation, free_positions,
        hits_snake, initial_snake, next_position, score_after_food,
    };

    #[test]
    fn direction_rejects_only_reverse_turns() {
        assert!(!Direction::Up.can_turn_to(Direction::Down));
        assert!(Direction::Up.can_turn_to(Direction::Left));
        assert!(Direction::Up.can_turn_to(Direction::Up));
    }

    #[test]
    fn movement_wraps_at_grid_edges() {
        assert_eq!(
            next_position(ivec2(GRID_WIDTH - 1, 0), Direction::Right),
            ivec2(0, 0)
        );
        assert_eq!(
            next_position(ivec2(0, 0), Direction::Up),
            ivec2(0, GRID_HEIGHT - 1)
        );
    }

    #[test]
    fn snake_starts_with_a_head_and_a_trailing_body_segment() {
        let start = ivec2(0, 0);
        let snake = initial_snake(start);
        assert_eq!(snake.len(), 2);
        assert_eq!(snake[0], start);
        assert_eq!(snake[1], ivec2(GRID_WIDTH - 1, 0));
    }

    #[test]
    fn collision_allows_vacating_tail_unless_snake_grows() {
        let snake = [ivec2(2, 2), ivec2(1, 2), ivec2(0, 2)];
        assert!(!hits_snake(&snake, ivec2(0, 2), false));
        assert!(hits_snake(&snake, ivec2(0, 2), true));
        assert!(hits_snake(&snake, ivec2(1, 2), false));
    }

    #[test]
    fn food_positions_exclude_snake_cells() {
        let snake = [ivec2(0, 0), ivec2(1, 0)];
        let free = free_positions(&snake);
        assert_eq!(free.len(), (GRID_WIDTH * GRID_HEIGHT - 2) as usize);
        assert!(!free.contains(&ivec2(0, 0)));
    }

    #[test]
    fn a_full_board_has_no_food_positions_left() {
        let snake: Vec<_> = (0..GRID_HEIGHT)
            .flat_map(|y| (0..GRID_WIDTH).map(move |x| ivec2(x, y)))
            .collect();
        assert!(free_positions(&snake).is_empty());
    }

    #[test]
    fn eating_food_awards_ten_points() {
        assert_eq!(FOOD_POINTS, 10);
        assert_eq!(score_after_food(0), 10);
        assert_eq!(score_after_food(30), 40);
        assert_eq!(score_after_food(u32::MAX), u32::MAX);
    }

    #[test]
    fn corner_turns_use_the_expected_flip() {
        let (rotation, flip) = corner_rotation(Direction::Up, Direction::Right);
        assert_eq!(rotation, 0.0);
        assert!(matches!(flip, Flip::Horizontal));
        let (rotation, flip) = corner_rotation(Direction::Up, Direction::Left);
        assert_eq!(rotation, 0.0);
        assert!(matches!(flip, Flip::None));
    }
}
