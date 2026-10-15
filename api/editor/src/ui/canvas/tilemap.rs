use super::{CanvasView, GridBackground, active_pointer, canvas_geometry};
use crate::palette;
use api::glam::UVec2;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Ui, Vec2};
use formats::{Tile, TilemapDocument, TilesetDocument};
use std::ops::Range;

const MAX_BATCH_VERTICES: usize = 32_768;

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

#[derive(Default)]
pub struct TilemapPreviewCache {
    fallback_colors: Vec<Option<Color32>>,
    tile_count: usize,
}

impl TilemapPreviewCache {
    pub fn clear(&mut self) {
        self.fallback_colors.fill(None);
    }
}

pub fn show_tilemap(
    ui: &mut Ui,
    document: &TilemapDocument,
    tileset: &TilesetDocument,
    placement: TilePlacement,
    view: &mut CanvasView,
    cache: &mut TilemapPreviewCache,
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
    let x_range = visible_cells(
        rect.left(),
        cell_size,
        width,
        viewport.left(),
        viewport.right(),
    );
    let y_range = visible_cells(
        rect.top(),
        cell_size,
        height,
        viewport.top(),
        viewport.bottom(),
    );
    if cache.tile_count != tileset.tiles.len() {
        cache.fallback_colors.clear();
        cache.fallback_colors.resize(tileset.tiles.len(), None);
        cache.tile_count = tileset.tiles.len();
    }

    // Paint cell backgrounds first so the tile-pixel meshes can be batched without
    // changing the layering of later cell outlines and labels.
    for y in y_range.clone() {
        for x in x_range.clone() {
            let index = y * width + x;
            let cell = document.cells.get(index).copied().unwrap_or_default();
            let cell_rect = cell_rect(rect, x, y, cell_size);
            if !cell_rect.intersects(viewport) {
                continue;
            }
            if cell.id == 0 {
                paint_checker(&painter, cell_rect, x, y, view.background);
            } else if let Some(tile) = tileset.tiles.get(cell.id.saturating_sub(1) as usize) {
                let fallback =
                    cache.fallback_colors[cell.id as usize - 1].get_or_insert_with(|| {
                        tile.pixels()
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
            } else {
                painter.rect_filled(cell_rect, 0.0, Color32::RED);
            }
        }
    }

    if can_preview_tiles {
        let tile_size = UVec2::new(tile_width as u32, tile_height as u32);
        let mut mesh = egui::Mesh::default();
        for y in y_range.clone() {
            for x in x_range.clone() {
                let index = y * width + x;
                let cell = document.cells.get(index).copied().unwrap_or_default();
                let Some(tile) = cell
                    .id
                    .checked_sub(1)
                    .and_then(|id| tileset.tiles.get(id as usize))
                else {
                    continue;
                };
                let rect = cell_rect(rect, x, y, cell_size);
                if !rect.intersects(viewport) {
                    continue;
                }
                paint_tile_pixels(&painter, &mut mesh, rect, tile.pixels(), tile_size, cell);
            }
        }
        if !mesh.is_empty() {
            painter.add(egui::Shape::mesh(mesh));
        }
    }

    for y in y_range.clone() {
        for x in x_range.clone() {
            let index = y * width + x;
            let cell = document.cells.get(index).copied().unwrap_or_default();
            let rect = cell_rect(rect, x, y, cell_size);
            if !rect.intersects(viewport) {
                continue;
            }
            painter.rect_stroke(
                rect,
                0.0,
                Stroke::new(0.5, view.background.grid_stroke()),
                egui::StrokeKind::Inside,
            );
            if cell_size >= 28.0 && cell.id > 0 {
                painter.text(
                    rect.left_top() + Vec2::splat(2.0),
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

fn cell_rect(canvas: Rect, x: usize, y: usize, cell_size: f32) -> Rect {
    Rect::from_min_size(
        Pos2::new(
            canvas.left() + x as f32 * cell_size,
            canvas.top() + y as f32 * cell_size,
        ),
        Vec2::splat(cell_size),
    )
}

fn visible_cells(
    origin: f32,
    cell_size: f32,
    count: usize,
    visible_start: f32,
    visible_end: f32,
) -> Range<usize> {
    if cell_size <= 0.0 || count == 0 {
        return 0..0;
    }
    let first = ((visible_start - origin) / cell_size)
        .floor()
        .clamp(0.0, count as f32) as usize;
    let end = ((visible_end - origin) / cell_size)
        .ceil()
        .clamp(0.0, count as f32) as usize;
    first..end.max(first)
}

fn paint_tile_pixels(
    painter: &egui::Painter,
    mesh: &mut egui::Mesh,
    rect: Rect,
    pixels: &[u8],
    size: UVec2,
    cell: Tile,
) {
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
                if mesh.vertices.len() >= MAX_BATCH_VERTICES {
                    painter.add(egui::Shape::mesh(std::mem::take(mesh)));
                }
                mesh.add_colored_rect(
                    Rect::from_min_size(
                        Pos2::new(
                            rect.left() + output_x as f32 * pixel_width,
                            rect.top() + output_y as f32 * pixel_height,
                        ),
                        Vec2::new(pixel_width, pixel_height),
                    ),
                    palette::color(color),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::visible_cells;

    #[test]
    fn visible_cell_ranges_include_partial_cells_and_clip_to_map_bounds() {
        assert_eq!(visible_cells(0.0, 10.0, 20, 25.0, 45.0), 2..5);
        assert_eq!(visible_cells(-15.0, 10.0, 20, 0.0, 20.0), 1..4);
        assert_eq!(visible_cells(0.0, 10.0, 20, 220.0, 240.0), 20..20);
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
