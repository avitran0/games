use std::{collections::HashMap, rc::Rc};

use formats::{AnimatedSpriteDocument, AnimationDirection, SpriteDocument, TilesetDocument};
use glam::UVec2;

use crate::render::gpu_image::GpuImage;

pub(crate) struct GlSprite {
    image: GpuImage,
    frame_count: usize,
    animations: HashMap<String, Animation>,
}

struct Animation {
    from: u16,
    to: u16,
    direction: AnimationDirection,
}

impl GlSprite {
    pub(crate) fn new_static(
        gl: Rc<glow::Context>,
        sprite: SpriteDocument,
    ) -> Result<Self, String> {
        let image = GpuImage::new_array(gl, sprite.pixels.pixels(), sprite.size, 1)?;
        Ok(Self {
            image,
            frame_count: 1,
            animations: HashMap::new(),
        })
    }

    pub(crate) fn new_animated(
        gl: Rc<glow::Context>,
        sprite: AnimatedSpriteDocument,
    ) -> Result<Self, String> {
        let data: Vec<u8> = sprite
            .frames
            .iter()
            .flat_map(|frame| frame.pixels().iter().copied())
            .collect();
        let frame_count = sprite.frames.len();
        let animations = sprite
            .tags
            .iter()
            .map(|tag| {
                (
                    tag.name.clone(),
                    Animation {
                        from: tag.start,
                        to: tag.end,
                        direction: tag.direction,
                    },
                )
            })
            .collect();
        let image = GpuImage::new_array(gl, &data, sprite.size, frame_count)?;

        Ok(Self {
            image,
            frame_count,
            animations,
        })
    }

    pub(crate) fn new_tileset(
        gl: Rc<glow::Context>,
        tileset: TilesetDocument,
    ) -> Result<Self, String> {
        let size = tileset.tile_size;
        let frame_count = tileset.tiles.len() + 1;
        let mut pixels = vec![0; (size.x * size.y) as usize];
        for tile in &tileset.tiles {
            pixels.extend_from_slice(tile.pixels());
        }
        let image = GpuImage::new_array(gl, &pixels, size, frame_count)?;
        Ok(Self {
            image,
            frame_count,
            animations: HashMap::new(),
        })
    }

    pub(crate) fn size(&self) -> UVec2 {
        self.image.size()
    }

    pub(crate) fn bind(&self) {
        self.image.bind();
    }

    pub(crate) fn frame_count(&self) -> usize {
        self.frame_count
    }

    pub(crate) fn animation_frame(&self, name: Option<&str>, tick: u16, divisor: u16) -> u32 {
        let Some(name) = name else {
            return 0;
        };
        let Some(animation) = self.animations.get(name) else {
            return 0;
        };
        animation.frame(tick / divisor.max(1)) as u32
    }
}

impl Animation {
    fn frame(&self, tick: u16) -> u16 {
        // use u32 because a valid u16 frame range can exceed the ping-pong period.
        let from = u32::from(self.from);
        let to = u32::from(self.to);
        let tick = u32::from(tick);
        let length = to - from + 1;
        if length <= 1 {
            return self.from;
        }

        (match self.direction {
            AnimationDirection::Forward => from + tick % length,
            AnimationDirection::Reverse => to - tick % length,
            AnimationDirection::PingPong => {
                let step = tick % (length * 2 - 2);
                if step < length {
                    from + step
                } else {
                    to - (step - length + 1)
                }
            }
            AnimationDirection::PingPongReverse => {
                let step = tick % (length * 2 - 2);
                if step < length {
                    to - step
                } else {
                    from + (step - length + 1)
                }
            }
        }) as u16
    }
}
