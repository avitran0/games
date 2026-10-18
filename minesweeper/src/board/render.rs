use api::{
    Anchor, Color, SpriteOptions,
    glam::{UVec2, Vec2, ivec2, uvec2},
};

use super::{Board, animation::FlagAnimation, delayed_tween::DelayedTween};
use crate::{
    game::TILE_SIZE,
    sprites::Sprites,
    tile::{Tile, TileKind},
};

#[derive(Clone, Copy)]
struct TileDraw<'a> {
    pos: UVec2,
    screen_pos: api::glam::IVec2,
    tile: Tile,
    sprites: &'a Sprites,
    scale: Vec2,
}

impl Board {
    pub fn draw(&self, camera: &api::Camera2D, world: &mut api::WorldFrame, sprites: &Sprites) {
        let (x_range, y_range) = self.visible_tile_ranges(camera);
        for y in y_range {
            for x in x_range.clone() {
                self.draw_tile(camera, world, uvec2(x, y), sprites);
            }
        }
    }

    fn visible_tile_ranges(
        &self,
        camera: &api::Camera2D,
    ) -> (std::ops::RangeInclusive<u32>, std::ops::RangeInclusive<u32>) {
        let top_left = camera.screen_to_world(ivec2(0, 0));
        let bottom_right = camera.screen_to_world(ivec2(
            camera.resolution().width as i32,
            camera.resolution().height as i32,
        ));
        let size = self.size();
        let x_range = Self::visible_tile_range(top_left.x, bottom_right.x, size.x);
        let y_range = Self::visible_tile_range(top_left.y, bottom_right.y, size.y);
        (x_range, y_range)
    }

    fn visible_tile_range(start: f32, end: f32, count: u32) -> std::ops::RangeInclusive<u32> {
        let max = count as i64 - 1;
        let first = ((start / TILE_SIZE as f32).floor() as i64 - 1).clamp(0, max) as u32;
        let last = ((end / TILE_SIZE as f32).floor() as i64 + 2).clamp(0, max) as u32;
        first..=last
    }

    fn draw_tile(
        &self,
        camera: &api::Camera2D,
        world: &mut api::WorldFrame<'_, '_>,
        pos: UVec2,
        sprites: &Sprites,
    ) {
        let index = Self::coord_to_index(pos, self.size.x as usize);
        let tile = self.tiles[index];
        let animation = &self.animations[index];
        let draw_pos = (pos * TILE_SIZE).as_ivec2();
        // fixes black bars that otherwise appear between sprites when zoomed out
        let screen_min = camera.world_to_screen(draw_pos.as_vec2());
        let screen_max = camera.world_to_screen((draw_pos + TILE_SIZE as i32).as_vec2());
        let screen_size = (screen_max - screen_min).as_vec2();
        let scale = screen_size / (TILE_SIZE as f32 * camera.zoom());
        let draw = TileDraw {
            pos,
            screen_pos: draw_pos,
            tile,
            sprites,
            scale,
        };
        let options = SpriteOptions::default().scale(scale);

        if !tile.uncovered {
            world.sprite(&sprites.covered, draw_pos, options);
            self.draw_flag(world, draw, animation.flag.as_ref());
            return;
        }

        let reveal_progress = animation
            .reveal
            .as_ref()
            .map(DelayedTween::current)
            .unwrap_or(1.0);
        if reveal_progress < 1.0 {
            world.sprite(&sprites.covered, draw_pos, options);
        }
        self.draw_uncovered(world, draw, reveal_progress);
    }

    fn draw_uncovered(
        &self,
        world: &mut api::WorldFrame<'_, '_>,
        draw: TileDraw<'_>,
        progress: f32,
    ) {
        let TileDraw {
            pos,
            screen_pos: draw_pos,
            tile,
            sprites,
            scale,
        } = draw;
        let center = draw_pos + TILE_SIZE as i32 / 2;
        let animating = progress < 1.0;
        let (sprite_position, options) = if animating {
            let pop_scale = 0.72 + 0.28 * progress;
            (
                center,
                SpriteOptions::default()
                    .anchor(Anchor::Center)
                    .scale(scale * Vec2::splat(pop_scale))
                    .opacity(progress),
            )
        } else {
            (draw_pos, SpriteOptions::default().scale(scale))
        };
        world.sprite(&sprites.uncovered, sprite_position, options);

        match tile.kind {
            TileKind::Clear => {
                let neighbors = self.neighbors(pos);
                if neighbors != 0 {
                    let text_pos = center;
                    let color = Self::neighbor_count_color(neighbors);
                    world.text_color_with_opacity(
                        neighbors.to_string(),
                        text_pos,
                        Anchor::Center,
                        color,
                        progress,
                    );
                }
            }
            TileKind::Bomb => {
                world.sprite(&sprites.bomb, sprite_position, options);
            }
        }
    }

    fn draw_flag(
        &self,
        world: &mut api::WorldFrame<'_, '_>,
        draw: TileDraw<'_>,
        animation: Option<&FlagAnimation>,
    ) {
        if !draw.tile.flagged {
            return;
        }

        let (scale_factor, opacity) = match animation {
            Some(animation) if !animation.tween.is_finished() => {
                let progress = animation.tween.current();
                (
                    0.55 + 0.45 * progress + 0.18 * (progress * std::f32::consts::PI).sin(),
                    progress,
                )
            }
            _ => (1.0, 1.0),
        };
        world.sprite(
            &draw.sprites.flag,
            draw.screen_pos + TILE_SIZE as i32 / 2,
            SpriteOptions::default()
                .anchor(Anchor::Center)
                .scale(draw.scale * Vec2::splat(scale_factor))
                .opacity(opacity),
        );
    }

    fn neighbor_count_color(neighbors: usize) -> Color {
        match neighbors {
            1 => Color::Indigo,
            2 => Color::ForestGreen,
            3 => Color::TallPoppy,
            4 => Color::PurpleHeart,
            _ => Color::Bunker,
        }
    }
}
