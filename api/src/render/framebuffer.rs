use std::rc::Rc;

use glam::ivec2;
use glow::HasContext;

use crate::{HEIGHT, WIDTH};

pub(crate) struct Framebuffer {
    gl: Rc<glow::Context>,
    fbo: glow::Framebuffer,
    texture: glow::Texture,
}

impl Framebuffer {
    pub(crate) fn new(gl: Rc<glow::Context>) -> Result<Self, String> {
        let texture = super::texture(&gl)?;
        let fbo = unsafe { gl.create_framebuffer()? };

        unsafe {
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA8 as i32,
                WIDTH as i32,
                HEIGHT as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(None),
            );

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(texture),
                0,
            );
            if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
                gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                gl.bind_texture(glow::TEXTURE_2D, None);
                gl.delete_framebuffer(fbo);
                gl.delete_texture(texture);

                return Err("The framebuffer is not complete.".to_owned());
            }
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
        Ok(Self { gl, fbo, texture })
    }

    pub(crate) fn bind(&self) {
        unsafe {
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo));
            self.gl.viewport(0, 0, WIDTH as i32, HEIGHT as i32);
        }
    }

    pub(crate) fn unbind(&self) {
        unsafe {
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
    }

    pub(crate) fn clear(&self) {
        unsafe {
            self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }
    }

    pub(crate) fn blit(&self, window_size: (u32, u32)) {
        let (width, height) = window_size;

        let scale = (width as f32 / WIDTH as f32).min(height as f32 / HEIGHT as f32);

        let size = ivec2(
            (WIDTH as f32 * scale).round() as i32,
            (HEIGHT as f32 * scale).round() as i32,
        );
        let pos = ivec2((width as i32 - size.x) / 2, (height as i32 - size.y) / 2);

        unsafe {
            self.gl
                .bind_framebuffer(glow::READ_FRAMEBUFFER, Some(self.fbo));
            self.gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, None);

            self.gl.blit_framebuffer(
                0,
                0,
                WIDTH as i32,
                HEIGHT as i32,
                pos.x,
                pos.y,
                pos.x + size.x,
                pos.y + size.y,
                glow::COLOR_BUFFER_BIT,
                glow::NEAREST,
            );

            self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_framebuffer(self.fbo);
            self.gl.delete_texture(self.texture);
        }
    }
}
