mod pixel;
mod tilemap;

pub use pixel::{PixelEdit, PixelMode, show_pixels};
pub use tilemap::{CellEdit, TilePlacement, show_tilemap};

use eframe::egui::{self, Color32, Pos2, Rect, Response, Sense, Ui, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridBackground {
    Light,
    Dark,
}

impl GridBackground {
    fn colors(self) -> (Color32, Color32) {
        match self {
            Self::Light => (Color32::from_gray(232), Color32::from_gray(195)),
            Self::Dark => (Color32::from_gray(58), Color32::from_gray(82)),
        }
    }

    fn grid_stroke(self) -> Color32 {
        match self {
            Self::Light => Color32::from_black_alpha(65),
            Self::Dark => Color32::from_white_alpha(55),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CanvasView {
    /// Zoom multiplier applied to the fit-to-view scale.
    zoom: f32,
    pan: Vec2,
    background: GridBackground,
}

impl Default for CanvasView {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
            background: GridBackground::Light,
        }
    }
}

pub fn controls(ui: &mut Ui, view: &mut CanvasView) {
    ui.heading("View");
    ui.horizontal(|ui| {
        if ui.button("-").on_hover_text("Zoom out").clicked() {
            view.zoom = (view.zoom / 1.25).clamp(0.25, 32.0);
        }
        if ui.button("+").on_hover_text("Zoom in").clicked() {
            view.zoom = (view.zoom * 1.25).clamp(0.25, 32.0);
        }
        if ui
            .button("Fit")
            .on_hover_text("Fit canvas to view")
            .clicked()
        {
            view.zoom = 1.0;
            view.pan = Vec2::ZERO;
        }
        ui.label(format!("{:.0}%", view.zoom * 100.0));
    });
    ui.horizontal(|ui| {
        ui.label("Grid");
        ui.selectable_value(&mut view.background, GridBackground::Light, "Light");
        ui.selectable_value(&mut view.background, GridBackground::Dark, "Dark");
    });
    ui.separator();
}

fn active_pointer(response: &Response, ui: &Ui) -> Option<(Pos2, bool)> {
    let pointer = response.interact_pointer_pos()?;
    let primary = !ui.input(|input| input.key_down(egui::Key::Space))
        && (response.clicked_by(egui::PointerButton::Primary)
            || response.dragged_by(egui::PointerButton::Primary));
    let secondary = response.clicked_by(egui::PointerButton::Secondary)
        || response.dragged_by(egui::PointerButton::Secondary);
    (primary || secondary).then_some((pointer, secondary))
}

fn canvas_geometry(
    ui: &mut Ui,
    size: api::glam::UVec2,
    view: &mut CanvasView,
) -> (Rect, Rect, f32, Response) {
    let available = ui.available_size().max(Vec2::splat(1.0));
    let (viewport, response) = ui.allocate_exact_size(available, Sense::click_and_drag());
    let width = size.x.max(1) as f32;
    let height = size.y.max(1) as f32;
    let fit_scale = ((viewport.width() - 40.0).max(1.0) / width)
        .min((viewport.height() - 40.0).max(1.0) / height)
        .floor()
        .max(1.0);

    let space_pan = ui.input(|input| input.key_down(egui::Key::Space))
        && response.dragged_by(egui::PointerButton::Primary);
    let middle_pan = response.dragged_by(egui::PointerButton::Middle);
    if middle_pan || space_pan {
        let delta = ui.input(|input| input.pointer.delta());
        view.pan += delta;
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
    } else if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
    }

    if response.hovered() {
        let scroll_y = ui.input(|input| input.smooth_scroll_delta.y);
        if scroll_y != 0.0
            && let Some(pointer) = response.hover_pos()
        {
            let old_zoom = view.zoom;
            let new_zoom = (old_zoom * 1.1_f32.powf(scroll_y / 40.0)).clamp(0.25, 32.0);
            let pointer_from_center = pointer - viewport.center();
            view.pan =
                pointer_from_center - (pointer_from_center - view.pan) * (new_zoom / old_zoom);
            view.zoom = new_zoom;
        }
    }

    let cell = fit_scale * view.zoom;
    let canvas_size = Vec2::new(width * cell, height * cell);
    let rect = Rect::from_center_size(viewport.center() + view.pan, canvas_size);
    (viewport, rect, cell, response)
}
