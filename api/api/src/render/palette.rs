use std::rc::Rc;

use crate::{formats::palette, render::gpu_image::GpuImage};

pub(crate) struct GlPalette {
    image: GpuImage,
}

impl GlPalette {
    pub(crate) fn new(gl: Rc<glow::Context>) -> Result<Self, String> {
        let image = GpuImage::new_single(gl, &palette::data(), crate::Color::COUNT as u32)?;

        Ok(Self { image })
    }

    pub(crate) fn bind(&self) {
        self.image.bind();
    }
}
