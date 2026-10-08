use std::rc::Rc;

use glam::{ivec2, vec2};
use glow::HasContext;

use crate::{
    Anchor, Flip, HEIGHT, Tilemap, WIDTH,
    assets::Assets,
    formats::palette::Palette,
    render::{
        draw_cmd::{AnimatedSpriteCmd, DrawCmd, ShapeCmd, SpriteCmd, TextCmd, TilemapCmd},
        framebuffer::Framebuffer,
        palette::GlPalette,
        quad::Quad,
        shader::Shader,
        sprite::GlSprite,
    },
};

pub(crate) mod draw_cmd;
pub(crate) mod font;
pub(crate) mod frame;
mod framebuffer;
pub(crate) mod gpu_image;
mod palette;
mod quad;
mod shader;
pub(crate) mod sprite;

pub(crate) struct Renderer {
    gl: Rc<glow::Context>,
    framebuffer: Framebuffer,
    shaders: Shaders,
    quad: Quad,
    palette: GlPalette,
}

impl Renderer {
    pub(crate) fn new(gl: Rc<glow::Context>) -> Result<Self, String> {
        let framebuffer = Framebuffer::new(gl.clone())?;
        let shaders = Shaders::load(gl.clone())?;
        let quad = Quad::new(gl.clone())?;
        let palette = GlPalette::new(gl.clone(), Palette)?;

        Ok(Self {
            gl,
            framebuffer,
            shaders,
            quad,
            palette,
        })
    }

    pub(crate) fn begin_frame(&self) {
        self.framebuffer.bind();
        self.framebuffer.clear();
    }

    pub(crate) fn end_frame(&self, window_size: (u32, u32)) {
        self.framebuffer.unbind();
        let (width, height) = window_size;
        unsafe {
            self.gl
                .viewport(0, 0, width.cast_signed(), height.cast_signed());
            self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }
        self.framebuffer.blit(window_size);
    }

    pub(crate) fn draw(&self, assets: &Assets, cmd: &DrawCmd) {
        match cmd {
            DrawCmd::Sprite(cmd) => self.draw_sprite(assets, cmd),
            DrawCmd::AnimatedSprite(cmd) => self.draw_animated_sprite(assets, cmd),
            DrawCmd::Tilemap(cmd) => self.draw_tilemap(assets, cmd),
            DrawCmd::Text(cmd) => self.draw_text(assets, cmd),
            DrawCmd::Shape(cmd) => self.draw_shape(cmd),
        }
    }

    fn draw_sprite(&self, assets: &Assets, cmd: &SpriteCmd) {
        let Some(sprite) = assets.get_sprite(cmd.sprite) else {
            return;
        };
        self.draw_sprite_frame(
            sprite,
            cmd.position,
            cmd.anchor,
            cmd.rotation,
            cmd.rotation_anchor,
            &cmd.flip,
            cmd.flip_diagonal,
            0,
        );
    }

    fn draw_animated_sprite(&self, assets: &Assets, cmd: &AnimatedSpriteCmd) {
        let Some(sprite) = assets.get_sprite(cmd.sprite) else {
            return;
        };
        let frame = sprite.animation_frame(
            cmd.animation.as_deref(),
            cmd.animation_tick,
            cmd.animation_divisor,
        );
        self.draw_sprite_frame(
            sprite,
            cmd.position,
            cmd.anchor,
            cmd.rotation,
            cmd.rotation_anchor,
            &cmd.flip,
            cmd.flip_diagonal,
            frame,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_sprite_frame(
        &self,
        sprite: &GlSprite,
        position: glam::IVec2,
        anchor: Anchor,
        rotation: f32,
        rotation_anchor: Anchor,
        flip: &Flip,
        flip_diagonal: bool,
        frame: u32,
    ) {
        let shader = &self.shaders.sprite;
        let size = sprite.size().as_vec2();
        shader.bind();
        shader.set_vec2("resolution", vec2(WIDTH as f32, HEIGHT as f32));
        shader.set_vec2("position", position.as_vec2() - anchor.offset(size));
        shader.set_vec2("size", size);
        shader.set_vec2("rotation_anchor", rotation_anchor.offset(size));
        shader.set_f32("rotation", rotation);
        shader.set_vec2("flip", flip.vector());
        shader.set_i32("flip_diagonal", i32::from(flip_diagonal));

        unsafe {
            self.gl.active_texture(glow::TEXTURE0);
        }
        sprite.bind();
        shader.set_i32("sprite_texture", 0);
        shader.set_i32("sprite_frame", frame as i32);

        unsafe {
            self.gl.active_texture(glow::TEXTURE1);
        }
        self.palette.bind();
        shader.set_i32("palette_texture", 1);

        self.quad.draw();
    }

    fn draw_tilemap(&self, assets: &Assets, cmd: &TilemapCmd) {
        let Some(tilemap) = assets.get_tilemap(Tilemap::new(cmd.tilemap)) else {
            return;
        };
        let Some(tileset_id) = assets.tileset_sprite(tilemap.tileset_id()) else {
            return;
        };
        let Some(tileset) = assets.get_sprite(tileset_id) else {
            return;
        };
        let tile_size = tileset.size().as_ivec2();

        let columns = usize::from(tilemap.size().x);
        let origin_x = i64::from(cmd.position.x);
        let origin_y = i64::from(cmd.position.y);
        let tile_width = i64::from(tile_size.x);
        let tile_height = i64::from(tile_size.y);
        let x_range = visible_tiles(origin_x, tile_width, columns, i64::from(WIDTH));
        let y_range = visible_tiles(
            origin_y,
            tile_height,
            usize::from(tilemap.size().y),
            i64::from(HEIGHT),
        );
        for y in y_range {
            for x in x_range.clone() {
                let tile = &tilemap.cells()[y * columns + x];
                if tile.id == 0 || tile.id as usize >= tileset.frame_count() {
                    continue;
                }
                let flip = match (tile.flip_x, tile.flip_y) {
                    (false, false) => Flip::None,
                    (true, false) => Flip::Horizontal,
                    (false, true) => Flip::Vertical,
                    (true, true) => Flip::Both,
                };
                self.draw_sprite_frame(
                    tileset,
                    ivec2(
                        (origin_x + x as i64 * tile_width) as i32,
                        (origin_y + y as i64 * tile_height) as i32,
                    ),
                    Anchor::TopLeft,
                    0.0,
                    Anchor::TopLeft,
                    &flip,
                    tile.flip_diagonal,
                    tile.id as u32,
                );
            }
        }
    }

    fn draw_shape(&self, cmd: &ShapeCmd) {
        if (cmd.color == crate::Color::Transparent
            && (cmd.border_color == crate::Color::Transparent || cmd.border_width <= 0.0))
            || cmd.size.x <= 0.0
            || cmd.size.y <= 0.0
        {
            return;
        }

        let shader = &self.shaders.shape;
        shader.bind();
        shader.set_vec2("resolution", vec2(WIDTH as f32, HEIGHT as f32));
        shader.set_vec2("position", cmd.position.as_vec2());
        shader.set_vec2("size", cmd.size);
        shader.set_vec2("rotation_anchor", vec2(0.0, 0.0));
        shader.set_f32("rotation", cmd.rotation);
        shader.set_i32("color_index", i32::from(cmd.color.index()));
        shader.set_f32("radius", cmd.radius.min(cmd.size.x.min(cmd.size.y) * 0.5));
        shader.set_i32("border_color_index", i32::from(cmd.border_color.index()));
        shader.set_f32("border_width", cmd.border_width);

        unsafe {
            self.gl.active_texture(glow::TEXTURE0);
        }
        self.palette.bind();
        shader.set_i32("palette_texture", 0);
        self.quad.draw();
    }

    fn draw_text(&self, assets: &Assets, cmd: &TextCmd) {
        let Some(font) = assets.get_font(cmd.font.unwrap_or(assets.default_font())) else {
            return;
        };

        let shader = &self.shaders.font;
        shader.bind();
        shader.set_vec2("resolution", vec2(WIDTH as f32, HEIGHT as f32));
        shader.set_i32("color_index", i32::from(cmd.color.index()));

        unsafe {
            self.gl.active_texture(glow::TEXTURE0);
        }
        font.bind();
        shader.set_i32("sprite_texture", 0);

        unsafe {
            self.gl.active_texture(glow::TEXTURE1);
        }
        self.palette.bind();
        shader.set_i32("palette_texture", 1);

        let lines: Vec<_> = cmd.text.split('\n').collect();
        let line_widths: Vec<i32> = lines
            .iter()
            .map(|line| {
                line.chars().fold(0, |width, c| {
                    let (glyph, _) = font.glyph(c);
                    width + glyph.advance as i32
                })
            })
            .collect();
        let size = ivec2(
            line_widths.iter().copied().max().unwrap_or(0),
            font.height() as i32 * lines.len() as i32,
        );

        let position = cmd.position;
        let pos = match cmd.anchor {
            Anchor::TopLeft => position,
            Anchor::Top => ivec2(position.x - size.x / 2, position.y),
            Anchor::TopRight => ivec2(position.x - size.x, position.y),
            Anchor::Left => ivec2(position.x, position.y - size.y / 2),
            Anchor::Center => ivec2(position.x - size.x / 2, position.y - size.y / 2),
            Anchor::Right => ivec2(position.x - size.x, position.y - size.y / 2),
            Anchor::BottomLeft => ivec2(position.x, position.y - size.y),
            Anchor::Bottom => ivec2(position.x - size.x / 2, position.y - size.y),
            Anchor::BottomRight => ivec2(position.x - size.x, position.y - size.y),
        };
        let center_lines = matches!(cmd.anchor, Anchor::Top | Anchor::Center | Anchor::Bottom);
        let right_align = matches!(
            cmd.anchor,
            Anchor::TopRight | Anchor::Right | Anchor::BottomRight
        );

        for (line_index, (line, line_width)) in lines.iter().zip(line_widths).enumerate() {
            let offset = if center_lines {
                (size.x - line_width) / 2
            } else if right_align {
                size.x - line_width
            } else {
                0
            };
            let mut line_pos = ivec2(
                pos.x + offset,
                pos.y + line_index as i32 * font.height() as i32,
            );

            for c in line.chars() {
                let (glyph, layer) = font.glyph(c);
                if glyph.width != 0 {
                    shader.set_vec2("position", line_pos.as_vec2());
                    shader.set_vec2("size", vec2(glyph.width as f32, font.height() as f32));
                    shader.set_i32("sprite_frame", layer as i32);
                    self.quad.draw();
                }
                line_pos.x += glyph.advance as i32;
            }
        }
    }
}

fn visible_tiles(
    origin: i64,
    tile_size: i64,
    count: usize,
    screen_size: i64,
) -> std::ops::Range<usize> {
    let count = count as i64;
    let first_visible = ((-origin).max(0) / tile_size).min(count);
    let end = if origin >= screen_size {
        0
    } else {
        ((screen_size - origin + tile_size - 1) / tile_size).min(count)
    };
    first_visible as usize..end.max(first_visible) as usize
}

struct Shaders {
    sprite: Shader,
    font: Shader,
    shape: Shader,
}

impl Shaders {
    fn load(gl: Rc<glow::Context>) -> Result<Self, String> {
        let sprite = Shader::load(
            gl.clone(),
            include_str!("shaders/sprite.vert"),
            include_str!("shaders/sprite.frag"),
        )?;
        let font = Shader::load(
            gl.clone(),
            include_str!("shaders/font.vert"),
            include_str!("shaders/font.frag"),
        )?;
        let shape = Shader::load(
            gl,
            include_str!("shaders/shape.vert"),
            include_str!("shaders/shape.frag"),
        )?;

        Ok(Self {
            sprite,
            font,
            shape,
        })
    }
}

fn texture(gl: &glow::Context) -> Result<glow::Texture, String> {
    texture_with_target(gl, glow::TEXTURE_2D)
}

fn texture_with_target(gl: &glow::Context, target: u32) -> Result<glow::Texture, String> {
    let texture = unsafe { gl.create_texture()? };
    unsafe {
        gl.bind_texture(target, Some(texture));
        gl.tex_parameter_i32(target, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
        gl.tex_parameter_i32(target, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
        gl.tex_parameter_i32(target, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
        gl.tex_parameter_i32(target, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
    }
    Ok(texture)
}
