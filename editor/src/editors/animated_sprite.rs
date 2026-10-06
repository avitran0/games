use crate::{
    document::AssetKind,
    editors::{FileState, apply_pixel_edit},
    ui::{self, animation_timeline, canvas::PixelMode},
};
use api::formats::{AnimatedSpriteDocument, AnimationDirection};
use eframe::egui::{self, Ui};

pub struct Screen {
    pub file: FileState,
    document: AnimatedSpriteDocument,
    state: State,
}

impl Screen {
    pub fn new(document: AnimatedSpriteDocument, file: FileState) -> Self {
        Self {
            file,
            document,
            state: State::default(),
        }
    }

    pub fn toolbar(&mut self, ui: &mut Ui) -> bool {
        toolbar(ui, &mut self.document, &mut self.state)
    }

    pub fn tools(&mut self, ui: &mut Ui) -> bool {
        tools(ui, &mut self.document, &mut self.state)
    }

    pub fn timeline(&mut self, ui: &mut Ui) -> bool {
        animation_timeline::show(ui, &mut self.document, &mut self.state.timeline)
    }

    pub fn canvas_controls(&mut self, ui: &mut Ui) {
        ui::canvas::controls(ui, &mut self.state.canvas);
    }

    pub fn canvas(&mut self, ui: &mut Ui) -> bool {
        canvas(ui, &mut self.document, &mut self.state)
    }

    pub fn save(&mut self, save_as: bool) {
        self.file.save(
            AssetKind::AnimatedSprite,
            save_as,
            self.document.encode().map_err(|error| error.to_string()),
        );
    }
}

struct State {
    selected_color: u8,
    timeline: animation_timeline::State,
    canvas: ui::canvas::CanvasView,
}

impl Default for State {
    fn default() -> Self {
        Self {
            selected_color: 1,
            timeline: animation_timeline::State::default(),
            canvas: ui::canvas::CanvasView::default(),
        }
    }
}

fn toolbar(ui: &mut Ui, document: &mut AnimatedSpriteDocument, state: &mut State) -> bool {
    let frame_count = document.frames.len();
    if frame_count == 0 {
        ui.label("Animation has no frames.");
        return false;
    }

    state.timeline.clamp_to(document);
    let mut changed = false;
    ui.horizontal(|ui| {
        let frame_index = state.timeline.frame_index();
        ui.label(format!("Frame {}/{}", frame_index + 1, frame_count));

        if ui.button("Previous").clicked() {
            state
                .timeline
                .select_frame((frame_index + frame_count - 1) % frame_count);
        }

        if ui.button("Next").clicked() {
            state.timeline.select_frame((frame_index + 1) % frame_count);
        }

        if ui
            .add_enabled(
                frame_count < u16::MAX as usize,
                egui::Button::new("Duplicate frame"),
            )
            .clicked()
        {
            animation_timeline::duplicate_frame(document, &mut state.timeline);
            changed = true;
        }

        if ui
            .add_enabled(frame_count > 1, egui::Button::new("Remove frame"))
            .clicked()
        {
            animation_timeline::remove_frame(document, &mut state.timeline);
            changed = true;
        }
    });

    changed
}

fn tools(ui: &mut Ui, document: &mut AnimatedSpriteDocument, state: &mut State) -> bool {
    if document.frames.is_empty() {
        ui.label("Animation has no frames.");
        return false;
    }

    if let Some(color) = ui::palette::show(ui, state.selected_color) {
        state.selected_color = color;
    }

    let mut changed = false;
    ui.add_space(8.0);
    ui.collapsing("Animation tags", |ui| {
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    document.tags.len() < u16::MAX as usize,
                    egui::Button::new("Add"),
                )
                .clicked()
            {
                animation_timeline::add_tag(document, &mut state.timeline);
                changed = true;
            }

            if ui
                .add_enabled(!document.tags.is_empty(), egui::Button::new("Remove"))
                .clicked()
            {
                animation_timeline::remove_tag(document, &mut state.timeline);
                changed = true;
            }
        });

        if document.tags.is_empty() {
            return;
        }

        state.timeline.clamp_to(document);
        let selected_tag = state.timeline.tag_index();
        for (index, tag) in document.tags.iter().enumerate() {
            if ui
                .selectable_label(
                    index == selected_tag,
                    format!(
                        "{}  [{}-{}]",
                        tag.name,
                        tag.start.saturating_add(1),
                        tag.end.saturating_add(1)
                    ),
                )
                .clicked()
            {
                state.timeline.select_tag(index);
            }
        }
        ui.separator();

        let frame_count = document.frames.len() as u16;
        let tag = &mut document.tags[state.timeline.tag_index()];
        ui.label("Name");
        changed |= ui.text_edit_singleline(&mut tag.name).changed();
        ui.horizontal(|ui| {
            ui.label("From");
            let mut first_frame = tag.start.saturating_add(1);
            let start_changed = ui
                .add(egui::DragValue::new(&mut first_frame).range(1..=frame_count))
                .changed();
            ui.label("To");
            let mut last_frame = tag.end.saturating_add(1);
            let end_changed = ui
                .add(egui::DragValue::new(&mut last_frame).range(1..=frame_count))
                .changed();
            if start_changed {
                tag.start = first_frame - 1;
            }
            if end_changed {
                tag.end = last_frame - 1;
            }
            if start_changed && tag.start > tag.end {
                tag.end = tag.start;
            } else if end_changed && tag.end < tag.start {
                tag.start = tag.end;
            }
            changed |= start_changed || end_changed;
        });

        egui::ComboBox::from_id_salt("tag-direction")
            .selected_text(direction_name(tag.direction))
            .show_ui(ui, |ui| {
                for direction in [
                    AnimationDirection::Forward,
                    AnimationDirection::Reverse,
                    AnimationDirection::PingPong,
                    AnimationDirection::PingPongReverse,
                ] {
                    changed |= ui
                        .selectable_value(&mut tag.direction, direction, direction_name(direction))
                        .changed();
                }
            });
    });
    changed
}

fn canvas(ui: &mut Ui, document: &mut AnimatedSpriteDocument, state: &mut State) -> bool {
    if document.frames.is_empty() {
        ui.label("Animation has no frames.");
        return false;
    }
    let frame_index = state.timeline.frame_index().min(document.frames.len() - 1);
    let frame = &mut document.frames[frame_index];
    let edit = ui::canvas::show_pixels(
        ui,
        frame.pixels(),
        document.size,
        state.selected_color,
        PixelMode::Indexed,
        &mut state.canvas,
    );
    apply_pixel_edit(frame.pixels_mut(), document.size, edit)
}

fn direction_name(direction: AnimationDirection) -> &'static str {
    match direction {
        AnimationDirection::Forward => "Forward",
        AnimationDirection::Reverse => "Reverse",
        AnimationDirection::PingPong => "Ping-pong",
        AnimationDirection::PingPongReverse => "Reverse ping-pong",
    }
}
