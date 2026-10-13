use crate::{
    document::AssetKind,
    editors::{FileState, apply_pixel_edit},
    ui::{self, canvas::PixelMode},
};
use api::glam::uvec2;
use eframe::egui::{self, Ui};
use formats::{FontDocument, GlyphDocument};

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

    pub fn preview_panel(&mut self, ui: &mut Ui) -> bool {
        preview_panel(ui, &mut self.document, &mut self.state)
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
    rename_text: String,
    rename_target: Option<(usize, char)>,
    rename_error: Option<String>,
    preview_text: String,
    canvas: ui::canvas::CanvasView,
}

fn toolbar(ui: &mut Ui, document: &mut FontDocument, state: &mut State) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        let count = document.glyphs.len();
        state.glyph_index = state.glyph_index.min(count.saturating_sub(1));

        ui.label("Width");
        let mut glyph_width = document.glyphs[state.glyph_index].width;
        if ui
            .add(egui::DragValue::new(&mut glyph_width).range(1..=u16::MAX))
            .changed()
        {
            resize_glyph_width(
                &mut document.glyphs[state.glyph_index],
                document.height,
                glyph_width,
            );
            changed = true;
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
            glyph.advance = glyph.width.saturating_add(1);
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
                        width: document.height,
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

fn resize_glyph_width(glyph: &mut GlyphDocument, height: u16, width: u16) {
    if glyph.width == width {
        return;
    }
    let old_width = usize::from(glyph.width);
    let new_width = usize::from(width);
    let height = usize::from(height);
    let mut bitmap = vec![0; new_width * height];
    let copy_width = old_width.min(new_width);
    for y in 0..height {
        let old_start = y * old_width;
        let new_start = y * new_width;
        bitmap[new_start..new_start + copy_width]
            .copy_from_slice(&glyph.bitmap[old_start..old_start + copy_width]);
    }
    glyph.width = width;
    glyph.bitmap = bitmap;
}

fn tools(ui: &mut Ui, document: &mut FontDocument, state: &mut State) -> bool {
    ui.heading("Font preview");
    ui.add(
        egui::TextEdit::singleline(&mut state.preview_text)
            .desired_width(ui.available_width())
            .hint_text("Type text to preview…"),
    );
    paint_font_preview(ui, document, &state.preview_text);
    ui.separator();

    let mut changed = false;
    let glyph = &document.glyphs[state.glyph_index];
    let target = (state.glyph_index, glyph.codepoint);
    if state.rename_target != Some(target) {
        state.rename_target = Some(target);
        state.rename_text = codepoint_label(glyph.codepoint);
        state.rename_error = None;
    }

    ui.heading("Glyph");
    ui.label(format!(
        "{} · {} px high",
        glyph_label(glyph.codepoint),
        document.height
    ));
    ui.horizontal(|ui| {
        ui.label("Code point");
        let response = ui
            .add(egui::TextEdit::singleline(&mut state.rename_text).desired_width(56.0))
            .on_hover_text("Enter a Unicode code point such as U+03B1.");
        let submit = ui.button("Rename").clicked()
            || (response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)));
        if response.changed() {
            state.rename_error = None;
        }
        if submit && let Some(codepoint) = parse_character(&state.rename_text) {
            if document
                .glyphs
                .iter()
                .enumerate()
                .any(|(index, glyph)| index != state.glyph_index && glyph.codepoint == codepoint)
            {
                state.rename_error = Some(format!("{} already exists.", glyph_label(codepoint)));
            } else {
                document.glyphs[state.glyph_index].codepoint = codepoint;
                state.rename_text = codepoint_label(codepoint);
                state.rename_target = Some((state.glyph_index, codepoint));
                state.rename_error = None;
                changed = true;
            }
        } else if submit {
            state.rename_error = Some("Enter a valid Unicode code point such as U+03B1.".into());
        }
    });
    if let Some(error) = &state.rename_error {
        ui.colored_label(ui.visuals().error_fg_color, error);
    }
    changed
}

fn preview_panel(ui: &mut Ui, document: &mut FontDocument, state: &mut State) -> bool {
    let mut reordered = false;
    ui.horizontal(|ui| {
        ui.strong("Glyphs");
        ui.label(format!("{} total", document.glyphs.len()));
        ui.separator();
        ui.label("Select a glyph to edit it.");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_enabled(
                    state.glyph_index + 1 < document.glyphs.len(),
                    egui::Button::new("Move right"),
                )
                .clicked()
            {
                document
                    .glyphs
                    .swap(state.glyph_index, state.glyph_index + 1);
                state.glyph_index += 1;
                reordered = true;
            }
            if ui
                .add_enabled(state.glyph_index > 0, egui::Button::new("Move left"))
                .clicked()
            {
                document
                    .glyphs
                    .swap(state.glyph_index, state.glyph_index - 1);
                state.glyph_index -= 1;
                reordered = true;
            }
        });
    });

    egui::ScrollArea::horizontal()
        .id_salt("font-glyph-preview")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let height = usize::from(document.height);
                for (index, glyph) in document.glyphs.iter().enumerate() {
                    const CARD_SIZE: egui::Vec2 = egui::Vec2::new(72.0, 96.0);
                    let (card, response) = ui.allocate_exact_size(CARD_SIZE, egui::Sense::click());
                    let painter = ui.painter_at(card);
                    let selected = index == state.glyph_index;
                    let fill = if selected {
                        ui.visuals().selection.bg_fill
                    } else {
                        ui.visuals().faint_bg_color
                    };
                    painter.rect_filled(card, 3.0, fill);

                    let glyph_width = usize::from(glyph.width);
                    let scale = ((CARD_SIZE.x - 8.0) / glyph_width as f32)
                        .min((CARD_SIZE.y - 30.0) / height as f32)
                        .clamp(0.1, 3.0);
                    let image_size =
                        egui::Vec2::new(glyph_width as f32 * scale, height as f32 * scale);
                    let image_rect = egui::Rect::from_center_size(
                        egui::Pos2::new(card.center().x, card.top() + 8.0 + image_size.y / 2.0),
                        image_size,
                    );
                    for y in 0..height {
                        for x in 0..glyph_width {
                            if glyph.bitmap[y * glyph_width + x] != 0 {
                                painter.rect_filled(
                                    egui::Rect::from_min_size(
                                        egui::Pos2::new(
                                            image_rect.left() + x as f32 * scale,
                                            image_rect.top() + y as f32 * scale,
                                        ),
                                        egui::Vec2::splat(scale),
                                    ),
                                    0.0,
                                    ui.visuals().text_color(),
                                );
                            }
                        }
                    }
                    painter.text(
                        egui::Pos2::new(card.center().x, card.bottom() - 11.0),
                        egui::Align2::CENTER_CENTER,
                        codepoint_label(glyph.codepoint),
                        egui::FontId::monospace(10.0),
                        ui.visuals().text_color(),
                    );
                    let stroke = if selected {
                        ui.visuals().selection.stroke
                    } else {
                        ui.visuals().widgets.noninteractive.bg_stroke
                    };
                    painter.rect_stroke(card, 3.0, stroke, egui::StrokeKind::Inside);
                    if response.clicked() {
                        state.glyph_index = index;
                    }
                    response.on_hover_text(format!(
                        "{} · advance {}",
                        glyph_label(glyph.codepoint),
                        glyph.advance
                    ));
                }
            });
        });
    reordered
}

fn canvas(ui: &mut Ui, document: &mut FontDocument, state: &mut State) -> bool {
    let height = document.height;
    let glyph = &mut document.glyphs[state.glyph_index];
    let size = uvec2(u32::from(glyph.width), u32::from(height));
    let edit = ui::canvas::show_pixels(
        ui,
        &glyph.bitmap,
        size,
        1,
        PixelMode::Monochrome,
        &mut state.canvas,
    );
    let edit = edit.and_then(|action| match action {
        ui::canvas::PixelAction::Paint(edit) => Some(edit),
        ui::canvas::PixelAction::PickColor(_) | ui::canvas::PixelAction::MoveSelection { .. } => {
            None
        }
    });
    apply_pixel_edit(&mut glyph.bitmap, size, edit)
}

fn paint_font_preview(ui: &mut Ui, document: &FontDocument, text: &str) {
    let scale = (44.0 / f32::from(document.height)).clamp(0.5, 3.0);
    let line_height = f32::from(document.height) * scale + 4.0;
    let mut line_width = 0.0_f32;
    let mut max_width = 0.0_f32;
    let mut lines = 1.0_f32;
    for character in text.chars() {
        if character == '\n' {
            max_width = max_width.max(line_width);
            line_width = 0.0;
            lines += 1.0;
        } else if character == '\t' {
            line_width += f32::from(document.height) * scale * 4.0;
        } else if character != '\r' {
            let glyph = find_glyph(document, character);
            line_width += f32::from(glyph.advance) * scale;
        }
    }
    max_width = max_width.max(line_width);
    let width = (max_width + 16.0).max(ui.available_width());
    let height = (lines * line_height + 8.0).max(40.0);
    egui::ScrollArea::horizontal()
        .id_salt("font-text-preview")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 2.0, ui.visuals().faint_bg_color);
            let mut x = rect.left() + 8.0;
            let mut y = rect.top() + 4.0;
            for character in text.chars() {
                match character {
                    '\n' => {
                        x = rect.left() + 8.0;
                        y += line_height;
                    }
                    '\t' => x += f32::from(document.height) * scale * 4.0,
                    '\r' => {}
                    character => {
                        let glyph = find_glyph(document, character);
                        let glyph_width = usize::from(glyph.width);
                        for row in 0..usize::from(document.height) {
                            for column in 0..glyph_width {
                                if glyph
                                    .bitmap
                                    .get(row * glyph_width + column)
                                    .copied()
                                    .unwrap_or(0)
                                    != 0
                                {
                                    painter.rect_filled(
                                        egui::Rect::from_min_size(
                                            egui::pos2(
                                                x + column as f32 * scale,
                                                y + row as f32 * scale,
                                            ),
                                            egui::Vec2::splat(scale),
                                        ),
                                        0.0,
                                        ui.visuals().text_color(),
                                    );
                                }
                            }
                        }
                        x += f32::from(glyph.advance) * scale;
                    }
                }
            }
        });
}

fn find_glyph(document: &FontDocument, character: char) -> &GlyphDocument {
    document
        .glyphs
        .iter()
        .find(|glyph| glyph.codepoint == character)
        .or_else(|| {
            document
                .glyphs
                .iter()
                .find(|glyph| glyph.codepoint == '\u{FFFD}')
        })
        .unwrap_or(&document.glyphs[0])
}

fn parse_character(text: &str) -> Option<char> {
    if let Some(hex) = text.strip_prefix("U+").or_else(|| text.strip_prefix("u+")) {
        return u32::from_str_radix(hex, 16).ok().and_then(char::from_u32);
    }
    let mut chars = text.chars();
    let result = chars.next()?;
    chars.next().is_none().then_some(result)
}

fn codepoint_label(codepoint: char) -> String {
    format!("U+{:04X}", codepoint as u32)
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

#[cfg(test)]
mod tests {
    use formats::GlyphDocument;

    use super::{parse_character, resize_glyph_width};

    #[test]
    fn parses_single_characters_and_unicode_values() {
        assert_eq!(parse_character("A"), Some('A'));
        assert_eq!(parse_character("U+1F600"), Some('😀'));
        assert_eq!(parse_character("u+0041"), Some('A'));
        assert_eq!(parse_character("U+D800"), None);
        assert_eq!(parse_character("AB"), None);
    }

    #[test]
    fn resizes_glyph_rows_without_shifting_pixels() {
        let mut glyph = GlyphDocument {
            codepoint: 'A',
            width: 2,
            advance: 2,
            bitmap: vec![1, 2, 3, 4],
        };
        resize_glyph_width(&mut glyph, 2, 3);
        assert_eq!(glyph.bitmap, [1, 2, 0, 3, 4, 0]);
        resize_glyph_width(&mut glyph, 2, 1);
        assert_eq!(glyph.bitmap, [1, 3]);
    }
}
