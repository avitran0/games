use std::rc::Rc;

use crate::{formats::palette::Palette, render::gpu_image::GpuImage};

pub(crate) struct GlPalette {
    image: GpuImage,
}

impl GlPalette {
    pub(crate) fn new(gl: Rc<glow::Context>, palette: Palette) -> Result<Self, String> {
        let image = GpuImage::new_single(gl, &palette.data(), 64)?;

        Ok(Self { image })
    }

    pub(crate) fn bind(&self) {
        self.image.bind();
    }
}
