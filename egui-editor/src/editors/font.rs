use crate::{
    document::AssetKind,
    editors::{FileState, apply_pixel_edit},
    ui::{self, canvas::PixelMode},
};
use api::formats::{FontDocument, GlyphDocument};
use api::glam::uvec2;
use eframe::egui::{self, Ui};

pub struct Screen {
    pub file: FileState,
    document: FontDocument,
    state: State,
}

impl Screen {
    pub fn new(document: FontDocument, file: FileState) -> Self {
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

    pub fn canvas_controls(&mut self, ui: &mut Ui) {
        ui::canvas::controls(ui, &mut self.state.canvas);
    }

    pub fn canvas(&mut self, ui: &mut Ui) -> bool {
        canvas(ui, &mut self.document, &mut self.state)
    }

    pub fn save(&mut self, save_as: bool) {
        self.file.save(
            AssetKind::Font,
            save_as,
            self.document.encode().map_err(|error| error.to_string()),
        );
    }
}

#[derive(Default)]
struct State {
    glyph_index: usize,
    add_text: String,
    preview_text: String,
    canvas: ui::canvas::CanvasView,
}

fn toolbar(ui: &mut Ui, document: &mut FontDocument, state: &mut State) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        let count = document.glyphs.len();
        state.glyph_index = state.glyph_index.min(count.saturating_sub(1));
        if ui.button("Previous").clicked() {
            state.glyph_index = (state.glyph_index + count - 1) % count;
        }
        egui::ComboBox::from_id_salt("font-glyph")
            .selected_text(glyph_label(document.glyphs[state.glyph_index].codepoint))
            .show_ui(ui, |ui| {
                for (index, glyph) in document.glyphs.iter().enumerate() {
                    ui.selectable_value(
                        &mut state.glyph_index,
                        index,
                        glyph_label(glyph.codepoint),
                    );
                }
            });
        if ui.button("Next").clicked() {
            state.glyph_index = (state.glyph_index + 1) % count;
        }

        ui.separator();
        ui.label("Advance");
        changed |= ui
            .add(egui::DragValue::new(
                &mut document.glyphs[state.glyph_index].advance,
            ))
            .changed();
        if ui.button("Fit").clicked() {
            let glyph = &mut document.glyphs[state.glyph_index];
            glyph.advance = glyph.width(document.height).saturating_add(1);
            changed = true;
        }

        ui.separator();
        ui.add_enabled_ui(count < u16::MAX as usize, |ui| {
            ui.text_edit_singleline(&mut state.add_text);
            if ui.button("Add glyph").clicked()
                && let Some(codepoint) = parse_character(&state.add_text)
            {
                if let Some(existing) = document
                    .glyphs
                    .iter()
                    .position(|glyph| glyph.codepoint == codepoint)
                {
                    state.glyph_index = existing;
                } else {
                    let side = usize::from(document.height);
                    document.glyphs.push(GlyphDocument {
                        codepoint,
                        advance: document.height,
                        bitmap: vec![0; side * side],
                    });
                    state.glyph_index = document.glyphs.len() - 1;
                    changed = true;
                }
                state.add_text.clear();
            }
        });
        if ui
            .add_enabled(count > 1, egui::Button::new("Remove glyph"))
            .clicked()
        {
            document.glyphs.remove(state.glyph_index);
            state.glyph_index = state.glyph_index.min(document.glyphs.len() - 1);
            changed = true;
        }
    });
    changed
}

fn tools(ui: &mut Ui, document: &mut FontDocument, state: &mut State) -> bool {
    ui.heading("Glyph");
    ui.label(format!(
        "{} - {} px high",
        glyph_label(document.glyphs[state.glyph_index].codepoint),
        document.height
    ));
    ui.separator();
    ui.label("Preview text");
    ui.text_edit_singleline(&mut state.preview_text);
    if !state.preview_text.is_empty() {
        draw_preview(ui, document, &state.preview_text);
    }
    false
}

fn canvas(ui: &mut Ui, document: &mut FontDocument, state: &mut State) -> bool {
    let glyph = &mut document.glyphs[state.glyph_index];
    let size = uvec2(u32::from(document.height), u32::from(document.height));
    let edit = ui::canvas::show_pixels(
        ui,
        &glyph.bitmap,
        size,
        1,
        PixelMode::Monochrome,
        &mut state.canvas,
    );
    apply_pixel_edit(&mut glyph.bitmap, size, edit)
}

fn parse_character(text: &str) -> Option<char> {
    if let Some(hex) = text.strip_prefix("U+").or_else(|| text.strip_prefix("u+")) {
        return u32::from_str_radix(hex, 16).ok().and_then(char::from_u32);
    }
    let mut chars = text.chars();
    let result = chars.next()?;
    chars.next().is_none().then_some(result)
}

fn glyph_label(codepoint: char) -> String {
    if codepoint == ' ' {
        "Space (U+0020)".into()
    } else if codepoint.is_ascii_graphic() {
        format!("'{codepoint}' (U+{:04X})", codepoint as u32)
    } else {
        format!("U+{:04X}", codepoint as u32)
    }
}

fn draw_preview(ui: &mut Ui, document: &FontDocument, text: &str) {
    let fallback = document
        .glyphs
        .iter()
        .find(|glyph| glyph.codepoint == '\u{FFFD}')
        .unwrap_or(&document.glyphs[0]);
    let glyphs: Vec<_> = text
        .chars()
        .take(32)
        .map(|codepoint| {
            document
                .glyphs
                .iter()
                .find(|glyph| glyph.codepoint == codepoint)
                .unwrap_or(fallback)
        })
        .collect();
    if glyphs.is_empty() {
        return;
    }

    let mut pen = 0_usize;
    let mut width = 1_usize;
    for glyph in &glyphs {
        width = width.max(pen + usize::from(glyph.width(document.height)));
        pen += usize::from(glyph.advance);
    }
    width = width.max(pen);
    let height = usize::from(document.height);
    let scale = (ui.available_width() / width as f32).clamp(0.1, 3.0);
    let size = egui::Vec2::new(width as f32 * scale, height as f32 * scale);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    pen = 0;
    for glyph in glyphs {
        for y in 0..height {
            for x in 0..height {
                if glyph.bitmap[y * height + x] == 0 {
                    continue;
                }
                painter.rect_filled(
                    egui::Rect::from_min_size(
                        egui::Pos2::new(
                            rect.left() + (pen + x) as f32 * scale,
                            rect.top() + y as f32 * scale,
                        ),
                        egui::Vec2::splat(scale),
                    ),
                    0.0,
                    ui.visuals().text_color(),
                );
            }
        }
        pen += usize::from(glyph.advance);
    }
}
