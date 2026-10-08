use super::{CanvasView, GridBackground, active_pointer, canvas_geometry};
use crate::palette;
use api::{
    formats::{Tile, TilemapDocument, TilesetDocument},
    glam::UVec2,
};
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Ui, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellEdit {
    pub index: usize,
    pub cell: Tile,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TilePlacement {
    pub selected_id: u32,
    pub flip_x: bool,
    pub flip_y: bool,
    pub flip_diagonal: bool,
}

pub fn show_tilemap(
    ui: &mut Ui,
    document: &TilemapDocument,
    tileset: &TilesetDocument,
    placement: TilePlacement,
    view: &mut CanvasView,
) -> Option<CellEdit> {
    let width = document.size.x as usize;
    let height = document.size.y as usize;
    let (viewport, rect, cell_size, response) = canvas_geometry(ui, document.size, view);
    let painter = ui.painter_at(viewport);
    let tile_width = tileset.tile_size.x as usize;
    let tile_height = tileset.tile_size.y as usize;
    let can_preview_tiles = cell_size >= tile_width.max(tile_height) as f32
        && width
            .saturating_mul(height)
            .saturating_mul(tile_width)
            .saturating_mul(tile_height)
            <= 1_000_000;
    let mut fallback_colors = vec![None; tileset.tiles.len()];

    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            let cell = document.cells.get(index).copied().unwrap_or_default();
            let cell_rect = Rect::from_min_size(
                Pos2::new(
                    rect.left() + x as f32 * cell_size,
                    rect.top() + y as f32 * cell_size,
                ),
                Vec2::splat(cell_size),
            );
            if !cell_rect.intersects(viewport) {
                continue;
            }
            if cell.id == 0 {
                paint_checker(&painter, cell_rect, x, y, view.background);
            } else if let Some(tile) = tileset.tiles.get(cell.id.saturating_sub(1) as usize) {
                let pixels = tile.pixels();
                let fallback = fallback_colors[cell.id as usize - 1].get_or_insert_with(|| {
                    pixels
                        .iter()
                        .copied()
                        .find(|pixel| *pixel != 0)
                        .map(palette::color)
                        .unwrap_or_else(|| {
                            palette::color(
                                ((cell.id - 1) % crate::palette::COLOR_COUNT as u32 + 1) as u8,
                            )
                        })
                });
                painter.rect_filled(cell_rect, 0.0, *fallback);
                if can_preview_tiles {
                    paint_tile_pixels(
                        &painter,
                        cell_rect,
                        pixels,
                        UVec2::new(tile_width as u32, tile_height as u32),
                        cell,
                    );
                }
            } else {
                painter.rect_filled(cell_rect, 0.0, Color32::RED);
            }

            painter.rect_stroke(
                cell_rect,
                0.0,
                Stroke::new(0.5, view.background.grid_stroke()),
                egui::StrokeKind::Inside,
            );
            if cell_size >= 28.0 && cell.id > 0 {
                painter.text(
                    cell_rect.left_top() + Vec2::splat(2.0),
                    egui::Align2::LEFT_TOP,
                    cell.id.to_string(),
                    egui::FontId::monospace((cell_size * 0.3).min(10.0)),
                    ui.visuals().text_color(),
                );
            }
        }
    }

    active_pointer(&response, ui).and_then(|(pointer, secondary)| {
        let x = ((pointer.x - rect.left()) / cell_size).floor() as usize;
        let y = ((pointer.y - rect.top()) / cell_size).floor() as usize;
        (rect.contains(pointer) && x < width && y < height).then_some(CellEdit {
            index: y * width + x,
            cell: if secondary || placement.selected_id == 0 {
                Tile::default()
            } else {
                Tile {
                    id: placement.selected_id,
                    flip_x: placement.flip_x,
                    flip_y: placement.flip_y,
                    flip_diagonal: placement.flip_diagonal,
                }
            },
        })
    })
}

fn paint_tile_pixels(painter: &egui::Painter, rect: Rect, pixels: &[u8], size: UVec2, cell: Tile) {
    let width = size.x as usize;
    let height = size.y as usize;
    let pixel_width = rect.width() / width as f32;
    let pixel_height = rect.height() / height as f32;
    for output_y in 0..height {
        for output_x in 0..width {
            let mut u = (output_x as f32 + 0.5) / width as f32;
            let mut v = (output_y as f32 + 0.5) / height as f32;
            if cell.flip_x {
                u = 1.0 - u;
            }
            if cell.flip_y {
                v = 1.0 - v;
            }
            if cell.flip_diagonal {
                std::mem::swap(&mut u, &mut v);
            }
            let source_x = ((u * width as f32) as usize).min(width - 1);
            let source_y = ((v * height as f32) as usize).min(height - 1);
            let color = pixels[source_y * width + source_x];
            if color != 0 {
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(
                            rect.left() + output_x as f32 * pixel_width,
                            rect.top() + output_y as f32 * pixel_height,
                        ),
                        Vec2::new(pixel_width, pixel_height),
                    ),
                    0.0,
                    palette::color(color),
                );
            }
        }
    }
}

fn paint_checker(
    painter: &egui::Painter,
    rect: Rect,
    x: usize,
    y: usize,
    background: GridBackground,
) {
    let (light, dark) = background.colors();
    painter.rect_filled(
        rect,
        0.0,
        if (x + y).is_multiple_of(2) {
            light
        } else {
            dark
        },
    );
}
