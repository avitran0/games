use glam::{IVec2, UVec2, Vec2, vec2};

use crate::{
    AnimatedSprite, Camera2D, Color, Font, Resolution, Sprite, Tilemap,
    render::draw_cmd::{AnimatedSpriteCmd, DrawCmd, ShapeCmd, SpriteCmd, TextCmd, TilemapCmd},
};

mod options;
mod world;

pub use options::{Anchor, Flip, ShapeOptions, SpriteOptions, TextOptions, TilemapOptions};
pub use world::WorldFrame;

#[derive(Default)]
pub struct Frame {
    tick: usize,
    resolution: Resolution,
    draw_cmds: Vec<DrawCmd>,
}

impl Frame {
    pub(crate) fn add(&mut self, cmd: DrawCmd) {
        self.draw_cmds.push(cmd);
    }

    pub(crate) fn cmds(&self) -> &[DrawCmd] {
        &self.draw_cmds
    }

    pub(crate) fn set_tick(&mut self, tick: usize) {
        self.tick = tick;
    }

    pub(crate) fn set_resolution(&mut self, resolution: crate::Resolution) {
        self.resolution = resolution;
    }

    pub(crate) fn clear(&mut self) {
        self.draw_cmds.clear();
    }

    pub fn tick(&self) -> usize {
        self.tick
    }

    pub fn resolution(&self) -> Resolution {
        self.resolution
    }

    pub fn width(&self) -> u32 {
        self.resolution.width
    }

    pub fn height(&self) -> u32 {
        self.resolution.height
    }

    pub fn world<'frame, 'camera>(
        &'frame mut self,
        camera: &'camera Camera2D,
    ) -> WorldFrame<'frame, 'camera> {
        WorldFrame {
            frame: self,
            camera,
        }
    }

    pub fn tilemap(&mut self, tilemap: &Tilemap, position: IVec2, options: TilemapOptions) {
        self.push_tilemap(tilemap, position, options);
    }

    pub(super) fn push_tilemap(
        &mut self,
        tilemap: &Tilemap,
        position: IVec2,
        options: TilemapOptions,
    ) {
        self.add(DrawCmd::Tilemap(TilemapCmd {
            tilemap: tilemap.id,
            position,
            scale: options.scale,
            opacity: options.opacity,
        }));
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
        self.push_shape(position, size.as_vec2(), color, options);
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
        self.push_shape(position, size.as_vec2(), color, options);
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
        self.push_shape(position, size.as_vec2(), color, options);
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

    pub(super) fn push_shape(
        &mut self,
        position: IVec2,
        size: Vec2,
        color: Color,
        options: ShapeOptions,
    ) {
        self.add(DrawCmd::Shape(ShapeCmd {
            position,
            size,
            rotation: 0.0,
            color,
            radius: options.radius,
            border_color: options.border_color,
            border_width: options.border_width,
            opacity: options.opacity,
        }));
    }

    pub fn line(&mut self, start: IVec2, end: IVec2, color: Color) {
        self.line_with_opacity(start, end, color, 1.0);
    }

    pub fn line_with_opacity(&mut self, start: IVec2, end: IVec2, color: Color, opacity: f32) {
        self.push_line(start, (end - start).as_vec2(), 1.0, color, opacity);
    }

    pub(super) fn push_line(
        &mut self,
        position: IVec2,
        delta: Vec2,
        thickness: f32,
        color: Color,
        opacity: f32,
    ) {
        let length = delta.length();
        self.add(DrawCmd::Shape(ShapeCmd {
            position,
            size: vec2(length.max(thickness), thickness),
            rotation: if length == 0.0 {
                0.0
            } else {
                delta.y.atan2(delta.x)
            },
            color,
            radius: 0.0,
            border_color: Color::Transparent,
            border_width: 0.0,
            opacity: opacity.clamp(0.0, 1.0),
        }));
    }

    pub fn sprite(&mut self, sprite: &Sprite, position: IVec2, options: SpriteOptions) {
        self.add(DrawCmd::Sprite(SpriteCmd {
            sprite: sprite.id,
            position,
            anchor: options.anchor,
            rotation: options.rotation.to_radians(),
            rotation_anchor: options.rotation_anchor,
            flip: options.flip,
            flip_diagonal: false,
            scale: options.scale,
            opacity: options.opacity,
        }));
    }

    pub fn animated_sprite(
        &mut self,
        sprite: &AnimatedSprite,
        position: IVec2,
        options: SpriteOptions,
    ) {
        self.add(DrawCmd::AnimatedSprite(AnimatedSpriteCmd {
            sprite: sprite.id,
            position,
            anchor: options.anchor,
            rotation: options.rotation.to_radians(),
            rotation_anchor: options.rotation_anchor,
            animation: sprite.animation().map(str::to_owned),
            animation_tick: sprite.animation_tick(),
            animation_divisor: sprite.animation_divisor(),
            flip: options.flip,
            flip_diagonal: false,
            scale: options.scale,
            opacity: options.opacity,
        }));
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
        self.push_text(text.into(), position, options);
    }

    fn push_text(&mut self, text: String, position: IVec2, options: TextOptions) {
        self.add(DrawCmd::Text(TextCmd {
            font: options.font,
            text,
            position,
            anchor: options.anchor,
            color: options.color,
            scale: options.scale,
            opacity: options.opacity,
        }));
    }
}
