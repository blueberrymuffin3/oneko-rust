use core::time;

use image::{DynamicImage, Frame, GenericImageView, RgbaImage, SubImage};

#[derive(Clone, Copy)]
pub struct FrameRef(pub u32, pub u32);

pub struct Animation {
    pub frames: &'static [FrameRef],
    pub interval: time::Duration,
}

pub struct SpriteSheet {
    image: RgbaImage,
    sprite_size: (u32, u32),
}

impl SpriteSheet {
    pub fn new(image: DynamicImage, sprite_size: (u32, u32)) -> Self {
        Self {
            image: image.into(),
            sprite_size,
        }
    }

    pub fn get_frame_view(&self, frame_ref: FrameRef) -> SubImage<&RgbaImage> {
        let FrameRef(x, y) = frame_ref;
        let (width, height) = self.sprite_size;
        let (x, y) = (x * width, y * height);

        return self.image.view(x, y, width, height);
    }

    pub fn get_size(&self) -> (u32, u32) {
        self.sprite_size
    }

    pub fn get_anim_view(&self, animation: &Animation, frame_count: usize) -> SubImage<&RgbaImage> {
        self.get_frame_view(animation.frames[frame_count % animation.frames.len()])
    }
}
