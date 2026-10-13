use std::rc::Rc;

use glam::{UVec2, uvec2};
use glow::HasContext;

pub(crate) struct GpuImage {
    gl: Rc<glow::Context>,
    texture: glow::Texture,
    target: u32,
    size: UVec2,
}

impl GpuImage {
    pub(crate) fn new_array(
        gl: Rc<glow::Context>,
        data: &[u8],
        size: UVec2,
        layers: usize,
    ) -> Result<Self, String> {
        let texture = super::texture_with_target(&gl, glow::TEXTURE_2D_ARRAY)?;
        unsafe {
            gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
            gl.tex_image_3d(
                glow::TEXTURE_2D_ARRAY,
                0,
                glow::R8UI as i32,
                size.x as i32,
                size.y as i32,
                layers as i32,
                0,
                glow::RED_INTEGER,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(data)),
            );
            gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 4);
            gl.bind_texture(glow::TEXTURE_2D_ARRAY, None);
        }
        Ok(Self {
            gl,
            texture,
            target: glow::TEXTURE_2D_ARRAY,
            size,
        })
    }

    pub(crate) fn new_single(
        gl: Rc<glow::Context>,
        data: &[u8],
        size: u32,
    ) -> Result<Self, String> {
        let texture = super::texture_with_target(&gl, glow::TEXTURE_2D)?;
        unsafe {
            gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGB8 as i32,
                size.cast_signed(),
                1,
                0,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(data)),
            );
            gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 4);
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
        Ok(Self {
            gl,
            texture,
            target: glow::TEXTURE_2D,
            size: uvec2(size, 1),
        })
    }

    pub(crate) fn size(&self) -> UVec2 {
        self.size
    }

    pub(crate) fn bind(&self) {
        unsafe {
            self.gl.bind_texture(self.target, Some(self.texture));
        }
    }
}

impl Drop for GpuImage {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_texture(self.texture);
        }
    }
}
