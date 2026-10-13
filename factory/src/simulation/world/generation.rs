use api::glam::{UVec2, uvec2};

use super::{World, terrain};
use crate::simulation::world::terrain::{GroundKind, OreKind};

pub(super) fn generate(world: &mut World) {
    let mut rng = api::Rng::seeded(world.seed ^ 0x9E37_79B9_7F4A_7C15);
    let center = world.size / 2;

    // Start with solid ground, then stamp irregular sandbars and shallow pools.
    for chunk in &mut world.chunks {
        for cell in &mut chunk.terrain {
            cell.ground = GroundKind::Stone;
        }
    }
    for _ in 0..(world.size.x * world.size.y / 300) {
        let origin = UVec2::new(
            rng.get_u32_range(0..world.size.x),
            rng.get_u32_range(0..world.size.y),
        );
        let radius = rng.get_u32_range(2..7);
        let ground = if rng.get_u8_range(0..5) == 0 {
            GroundKind::Water
        } else {
            GroundKind::Sand
        };
        for y in origin.y.saturating_sub(radius)..=(origin.y + radius).min(world.size.y - 1) {
            for x in origin.x.saturating_sub(radius)..=(origin.x + radius).min(world.size.x - 1) {
                let dx = x.abs_diff(origin.x) as i64;
                let dy = y.abs_diff(origin.y) as i64;
                let edge_noise = i64::from(rng.get_u8_range(0..4)) - 1;
                let distance = dx * dx + dy * dy;
                let position = uvec2(x, y);
                let spawn_distance = position
                    .x
                    .abs_diff(center.x)
                    .max(position.y.abs_diff(center.y));
                if distance <= (radius * radius) as i64 + edge_noise
                    && spawn_distance > 12
                    && let Some(coords) = world.locate(position)
                {
                    world.chunks[coords.chunk].terrain[coords.cell].ground = ground;
                }
            }
        }
    }

    for y in center.y.saturating_sub(4)..=(center.y + 4).min(world.size.y - 1) {
        for x in center.x.saturating_sub(4)..=(center.x + 4).min(world.size.x - 1) {
            let dx = x.abs_diff(center.x) as i32;
            let dy = y.abs_diff(center.y) as i32;
            if dx * dx + dy * dy <= 16 {
                set_ore(world, uvec2(x, y), OreKind::Iron);
            }
        }
    }

    for _ in 0..(world.size.x * world.size.y / 180) {
        let origin = UVec2::new(
            rng.get_u32_range(0..world.size.x),
            rng.get_u32_range(0..world.size.y),
        );
        let radius = rng.get_u32_range(2..5);
        let ore = match rng.get_u8_range(0..10) {
            0..=5 => OreKind::Iron,
            6..=8 => OreKind::Coal,
            _ => OreKind::Titanium,
        };
        for y in origin.y.saturating_sub(radius)..=(origin.y + radius).min(world.size.y - 1) {
            for x in origin.x.saturating_sub(radius)..=(origin.x + radius).min(world.size.x - 1) {
                let dx = x.abs_diff(origin.x) as i64;
                let dy = y.abs_diff(origin.y) as i64;
                if dx * dx + dy * dy <= (radius * radius) as i64
                    && (uvec2(x, y).as_ivec2() - center.as_ivec2()).length_squared() > 100
                {
                    set_ore(world, uvec2(x, y), ore);
                }
            }
        }
    }
}

fn set_ore(world: &mut World, position: UVec2, ore: OreKind) {
    let Some(coords) = world.locate(position) else {
        return;
    };
    let cell = &mut world.chunks[coords.chunk].terrain[coords.cell];
    if !matches!(cell.ground, terrain::GroundKind::Water) {
        cell.ore = Some(ore);
    }
}
