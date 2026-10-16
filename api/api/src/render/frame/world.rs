use glam::{IVec2, UVec2};

use crate::{AnimatedSprite, Camera2D, Color, Font, Sprite, Tilemap};

use super::{Anchor, Frame, ShapeOptions, SpriteOptions, TextOptions, TilemapOptions};

pub struct WorldFrame<'frame, 'camera> {
    pub(super) frame: &'frame mut Frame,
    pub(super) camera: &'camera Camera2D,
}

impl WorldFrame<'_, '_> {
    fn screen_position(&self, position: IVec2) -> IVec2 {
        self.camera.world_to_screen(position.as_vec2())
    }

    pub fn tilemap(&mut self, tilemap: &Tilemap, position: IVec2) {
        self.tilemap_with_options(tilemap, position, TilemapOptions::default());
    }

    pub fn tilemap_with_options(
        &mut self,
        tilemap: &Tilemap,
        position: IVec2,
        options: TilemapOptions,
    ) {
        self.frame.push_tilemap(
            tilemap,
            self.screen_position(position),
            options.scaled(self.camera.zoom()),
        );
    }

    pub fn rectangle(&mut self, position: IVec2, size: UVec2, color: Color) {
        self.rectangle_with_opacity(position, size, color, 1.0);
    }

    pub fn rectangle_with_opacity(
        &mut self,
        position: IVec2,
        size: UVec2,
        color: Color,
        opacity: f32,
    ) {
        self.rectangle_with_options(
            position,
            size,
            color,
            ShapeOptions::default().opacity(opacity),
        );
    }

    pub fn rectangle_with_options(
        &mut self,
        position: IVec2,
        size: UVec2,
        color: Color,
        options: ShapeOptions,
    ) {
        self.shape(position, size, color, options);
    }

    pub fn rounded_rectangle(&mut self, position: IVec2, size: UVec2, radius: u32, color: Color) {
        self.rounded_rectangle_with_options(
            position,
            size,
            color,
            ShapeOptions::default().radius(radius),
        );
    }

    pub fn rounded_rectangle_with_opacity(
        &mut self,
        position: IVec2,
        size: UVec2,
        radius: u32,
        color: Color,
        opacity: f32,
    ) {
        self.rounded_rectangle_with_options(
            position,
            size,
            color,
            ShapeOptions::default().radius(radius).opacity(opacity),
        );
    }

    pub fn rounded_rectangle_with_options(
        &mut self,
        position: IVec2,
        size: UVec2,
        color: Color,
        options: ShapeOptions,
    ) {
        self.shape(position, size, color, options);
    }

    pub fn bordered_rectangle(
        &mut self,
        position: IVec2,
        size: UVec2,
        radius: u32,
        color: Color,
        border_color: Color,
        border_width: u32,
    ) {
        self.bordered_rectangle_with_options(
            position,
            size,
            color,
            ShapeOptions::default()
                .radius(radius)
                .border(border_color, border_width),
        );
    }

    pub fn bordered_rectangle_with_options(
        &mut self,
        position: IVec2,
        size: UVec2,
        color: Color,
        options: ShapeOptions,
    ) {
        self.shape(position, size, color, options);
    }

    fn shape(&mut self, position: IVec2, size: UVec2, color: Color, options: ShapeOptions) {
        let zoom = self.camera.zoom();
        self.frame.push_shape(
            self.screen_position(position),
            size.as_vec2() * zoom,
            color,
            options.scaled(zoom),
        );
    }

    pub fn rounded_rectangle_outline(
        &mut self,
        position: IVec2,
        size: UVec2,
        radius: u32,
        border_color: Color,
        border_width: u32,
    ) {
        self.bordered_rectangle(
            position,
            size,
            radius,
            Color::Transparent,
            border_color,
            border_width,
        );
    }

    pub fn line(&mut self, start: IVec2, end: IVec2, color: Color) {
        self.line_with_opacity(start, end, color, 1.0);
    }

    pub fn line_with_opacity(&mut self, start: IVec2, end: IVec2, color: Color, opacity: f32) {
        let zoom = self.camera.zoom();
        let delta = (end - start).as_vec2() * zoom;
        self.frame
            .push_line(self.screen_position(start), delta, zoom, color, opacity);
    }

    pub fn sprite(&mut self, sprite: &Sprite, position: IVec2, options: SpriteOptions) {
        let options = options.scaled(self.camera.zoom());
        self.frame
            .sprite(sprite, self.screen_position(position), options);
    }

    pub fn animated_sprite(
        &mut self,
        sprite: &AnimatedSprite,
        position: IVec2,
        options: SpriteOptions,
    ) {
        let options = options.scaled(self.camera.zoom());
        self.frame
            .animated_sprite(sprite, self.screen_position(position), options);
    }

    pub fn text(&mut self, text: impl Into<String>, position: IVec2, anchor: Anchor) {
        self.text_with_options(text, position, TextOptions::default().anchor(anchor));
    }

    pub fn text_with_opacity(
        &mut self,
        text: impl Into<String>,
        position: IVec2,
        anchor: Anchor,
        opacity: f32,
    ) {
        self.text_with_options(
            text,
            position,
            TextOptions::default().anchor(anchor).opacity(opacity),
        );
    }

    pub fn text_font(
        &mut self,
        font: Font,
        text: impl Into<String>,
        position: IVec2,
        anchor: Anchor,
    ) {
        self.text_with_options(
            text,
            position,
            TextOptions::default().font(font).anchor(anchor),
        );
    }

    pub fn text_font_with_opacity(
        &mut self,
        font: Font,
        text: impl Into<String>,
        position: IVec2,
        anchor: Anchor,
        opacity: f32,
    ) {
        self.text_with_options(
            text,
            position,
            TextOptions::default()
                .font(font)
                .anchor(anchor)
                .opacity(opacity),
        );
    }

    pub fn text_color(
        &mut self,
        text: impl Into<String>,
        position: IVec2,
        anchor: Anchor,
        color: Color,
    ) {
        self.text_with_options(
            text,
            position,
            TextOptions::default().anchor(anchor).color(color),
        );
    }

    pub fn text_color_with_opacity(
        &mut self,
        text: impl Into<String>,
        position: IVec2,
        anchor: Anchor,
        color: Color,
        opacity: f32,
    ) {
        self.text_with_options(
            text,
            position,
            TextOptions::default()
                .anchor(anchor)
                .color(color)
                .opacity(opacity),
        );
    }

    pub fn text_with_options(
        &mut self,
        text: impl Into<String>,
        position: IVec2,
        options: TextOptions,
    ) {
        let options = options.scaled(self.camera.zoom());
        self.frame
            .push_text(text.into(), self.screen_position(position), options);
    }
}
