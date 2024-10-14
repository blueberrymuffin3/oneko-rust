use std::{
    sync::OnceLock,
    time::{self, Duration},
};

use image::{imageops::FilterType, RgbaImage, SubImage};
use rand::seq::SliceRandom;

use crate::sprite_sheet::{Animation, FrameRef, SpriteSheet};

const SCALE: u32 = 2;
const SPEED: f64 = 10.0 * (SCALE as f64);
const FOLLOW_DISTANCE: f64 = 60.0 * (SCALE as f64);

const ONEKO_IMG_DATA: &[u8] = include_bytes!("./maia_oneko.gif");
static SPRITE_SHEET: OnceLock<SpriteSheet> = OnceLock::new();

pub struct Oneko {
    anim: AnimState,
    frame_count: u32,
    offset: (i32, i32),
}

impl Default for Oneko {
    fn default() -> Self {
        SPRITE_SHEET.get_or_init(|| {
            let image =
                image::load_from_memory(ONEKO_IMG_DATA).expect("Error loading spritesheet image");

            let image = image.resize(
                image.width() * SCALE,
                image.height() * SCALE,
                FilterType::Nearest,
            );

            SpriteSheet::new(image, (32 * SCALE, 32 * SCALE))
        });

        // let mut rng = rand::thread_rng();
        // let offset = (rng.gen_range(-50..=50), rng.gen_range(-50..=50));

        Self {
            anim: AnimState::Idle(AnimStateIdle::Idle),
            frame_count: 0,
            offset: (0, 0),
        }
    }
}

impl Oneko {
    pub fn act(
        &mut self,
        (cat_x, cat_y): (i32, i32),
        (mouse_x, mouse_y): (i32, i32),
        (monitor_width, monitor_height): (i32, i32),
    ) -> (time::Duration, (i32, i32)) {
        let (offset_x, offset_y) = self.offset;
        let (cat_width, cat_height) = SPRITE_SHEET.get().unwrap().get_size();
        let (cat_width, cat_height) = (cat_width as i32, cat_height as i32);
        let cat_cx = cat_x + cat_width / 2;
        let cat_cy = cat_y + cat_height / 2;

        let mouse_dx = mouse_x + offset_x - cat_cx;
        let mouse_dy = mouse_y + offset_y - cat_cy;

        let mouse_dxf: f64 = mouse_dx.into();
        let mouse_dyf: f64 = mouse_dy.into();
        let distance = f64::sqrt(mouse_dxf * mouse_dxf + mouse_dyf * mouse_dyf);

        let (_, touching_wall, mut scratch_anim) = [
            ((0, -1), AnimStateScratch::ScratchWallN),
            ((1, 0), AnimStateScratch::ScratchWallE),
            ((0, 1), AnimStateScratch::ScratchWallS),
            ((-1, 0), AnimStateScratch::ScratchWallW),
        ]
        .into_iter()
        .map(|((x, y), anim)| {
            let mouse_dist = mouse_dx * x + mouse_dy * y;
            let touching = match (x, y) {
                (0, -1) => cat_y <= 0,
                (1, 0) => cat_x >= monitor_width - cat_width,
                (0, 1) => cat_y >= monitor_height - cat_height,
                (-1, 0) => cat_x <= 0,
                _ => unreachable!(),
            };
            (mouse_dist, touching, anim)
        })
        .max_by_key(|(mouse_dist, _, _)| *mouse_dist)
        .unwrap();

        if !touching_wall {
            scratch_anim = AnimStateScratch::ScratchSelf;
        }

        let active = distance > FOLLOW_DISTANCE && !touching_wall;

        let next_moving_state: AnimState =
            AnimState::Moving(AnimStateMoving::from_vector((mouse_dx, mouse_dy)));
        let next_anim = match self.anim {
            AnimState::Moving(..) if !active => AnimState::Idle(AnimStateIdle::Idle),
            AnimState::Moving(..) => next_moving_state,
            AnimState::Idle(..) if active => AnimState::Alert,
            // Pick something to do
            AnimState::Idle(AnimStateIdle::Idle) if self.frame_count > 10 => AnimState::Idle(
                [
                    AnimStateIdle::Scratch(scratch_anim),
                    AnimStateIdle::Tired,
                ]
                .choose(&mut rand::thread_rng())
                .unwrap()
                .clone(),
            ),
            // Done scratching
            AnimState::Idle(AnimStateIdle::Scratch(..)) if self.frame_count > 9 => {
                AnimState::Idle(AnimStateIdle::Idle)
            }
            AnimState::Idle(AnimStateIdle::Tired) if self.frame_count > 7 => {
                AnimState::Idle(AnimStateIdle::Sleeping)
            }
            AnimState::Idle(AnimStateIdle::Sleeping) if self.frame_count > 46 => {
                AnimState::Idle(AnimStateIdle::Idle)
            }
            AnimState::Alert if self.frame_count > 6 => next_moving_state,
            _ => self.anim,
        };

        if next_anim != self.anim {
            self.anim = next_anim;
            self.frame_count = 0;
        } else {
            self.frame_count += 1;
        }

        let animation = get_animation(self.anim);
        let (mut delta_x, mut delta_y) = match next_anim {
            AnimState::Moving(..) => (
                (mouse_dxf / distance * SPEED) as i32,
                (mouse_dyf / distance * SPEED) as i32,
            ),
            _ => (0, 0),
        };
        if delta_x.abs() > mouse_dx.abs() {
            delta_x = mouse_dx;
        }
        if delta_y.abs() > mouse_dy.abs() {
            delta_y = mouse_dy;
        }

        let cat_x = (cat_x + delta_x).clamp(0, monitor_width - cat_width);
        let cat_y = (cat_y + delta_y).clamp(0, monitor_width - cat_width);

        return (animation.interval, (cat_x, cat_y));
    }

    pub fn click(&mut self) {
        self.anim = AnimState::Alert;
        self.frame_count = 0;
    }

    pub fn get_frame(&self) -> SubImage<&RgbaImage> {
        let animation = get_animation(self.anim);
        return SPRITE_SHEET
            .get()
            .unwrap()
            .get_anim_view(&animation, self.frame_count as usize);
    }

    // pub fn get_icon(&self, size: u32) -> Icon {
    //     let image = SPRITE_SHEET
    //         .get()
    //         .unwrap()
    //         .get_frame_view(FrameRef(3, 3))
    //         .to_image();
    //     let image = DynamicImage::from(image)
    //         .resize(size, size, FilterType::Nearest)
    //         .into_rgba8();
    //     let (width, height) = image.dimensions();
    //     let image_data = image.into_raw();
    //     Icon::from_rgba(image_data, width, height).expect("Error creating icon")
    // }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnimState {
    Idle(AnimStateIdle),
    Alert,
    Moving(AnimStateMoving),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnimStateIdle {
    Idle,
    Scratch(AnimStateScratch),
    Tired,
    Sleeping,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnimStateScratch {
    ScratchSelf,
    ScratchWallN,
    ScratchWallS,
    ScratchWallE,
    ScratchWallW,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnimStateMoving {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl AnimStateMoving {
    fn from_vector((x, y): (i32, i32)) -> Self {
        enum AxisDirection {
            Neg,
            Zero,
            Pos,
        }

        let dx: f64 = x.into();
        let dy: f64 = y.into();
        let distance: f64 = f64::sqrt(dx * dx + dy * dy);

        let x_direction = match dx / distance {
            ..-0.5 => AxisDirection::Neg,
            0.5.. => AxisDirection::Pos,
            _ => AxisDirection::Zero,
        };

        let y_direction = match dy / distance {
            ..-0.5 => AxisDirection::Neg,
            0.5.. => AxisDirection::Pos,
            _ => AxisDirection::Zero,
        };

        match (x_direction, y_direction) {
            (AxisDirection::Zero, AxisDirection::Neg) => AnimStateMoving::N,
            (AxisDirection::Pos, AxisDirection::Neg) => AnimStateMoving::NE,
            (AxisDirection::Pos, AxisDirection::Zero) => AnimStateMoving::E,
            (AxisDirection::Pos, AxisDirection::Pos) => AnimStateMoving::SE,
            (AxisDirection::Zero, AxisDirection::Pos) => AnimStateMoving::S,
            (AxisDirection::Neg, AxisDirection::Pos) => AnimStateMoving::SW,
            (AxisDirection::Neg, AxisDirection::Zero) => AnimStateMoving::W,
            (AxisDirection::Neg, AxisDirection::Neg) => AnimStateMoving::NW,
            (AxisDirection::Zero, AxisDirection::Zero) => AnimStateMoving::E, // Pick a direction
        }
    }
}

fn get_animation(state: AnimState) -> Animation {
    match state {
        AnimState::Idle(AnimStateIdle::Idle) => Animation {
            frames: &[FrameRef(3, 3)],
            interval: Duration::from_millis(100),
        },
        AnimState::Idle(AnimStateIdle::Scratch(AnimStateScratch::ScratchSelf)) => Animation {
            frames: &[FrameRef(5, 0), FrameRef(6, 0), FrameRef(7, 0)],
            interval: Duration::from_millis(100),
        },
        AnimState::Idle(AnimStateIdle::Scratch(AnimStateScratch::ScratchWallN)) => Animation {
            frames: &[FrameRef(0, 0), FrameRef(0, 1)],
            interval: Duration::from_millis(100),
        },
        AnimState::Idle(AnimStateIdle::Scratch(AnimStateScratch::ScratchWallS)) => Animation {
            frames: &[FrameRef(7, 1), FrameRef(6, 2)],
            interval: Duration::from_millis(100),
        },
        AnimState::Idle(AnimStateIdle::Scratch(AnimStateScratch::ScratchWallE)) => Animation {
            frames: &[FrameRef(2, 2), FrameRef(2, 3)],
            interval: Duration::from_millis(100),
        },
        AnimState::Idle(AnimStateIdle::Scratch(AnimStateScratch::ScratchWallW)) => Animation {
            frames: &[FrameRef(4, 0), FrameRef(4, 1)],
            interval: Duration::from_millis(100),
        },
        AnimState::Idle(AnimStateIdle::Tired) => Animation {
            frames: &[FrameRef(3, 2)],
            interval: Duration::from_millis(100),
        },
        AnimState::Idle(AnimStateIdle::Sleeping) => Animation {
            frames: &[FrameRef(2, 0), FrameRef(2, 1)],
            interval: Duration::from_millis(400),
        },
        AnimState::Alert => Animation {
            frames: &[FrameRef(7, 3)],
            interval: Duration::from_millis(100),
        },
        AnimState::Moving(AnimStateMoving::N) => Animation {
            frames: &[FrameRef(1, 2), FrameRef(1, 3)],
            interval: Duration::from_millis(100),
        },
        AnimState::Moving(AnimStateMoving::NE) => Animation {
            frames: &[FrameRef(0, 2), FrameRef(0, 3)],
            interval: Duration::from_millis(100),
        },
        AnimState::Moving(AnimStateMoving::E) => Animation {
            frames: &[FrameRef(3, 0), FrameRef(3, 1)],
            interval: Duration::from_millis(100),
        },
        AnimState::Moving(AnimStateMoving::SE) => Animation {
            frames: &[FrameRef(5, 1), FrameRef(5, 2)],
            interval: Duration::from_millis(100),
        },
        AnimState::Moving(AnimStateMoving::S) => Animation {
            frames: &[FrameRef(6, 3), FrameRef(7, 2)],
            interval: Duration::from_millis(100),
        },
        AnimState::Moving(AnimStateMoving::SW) => Animation {
            frames: &[FrameRef(5, 3), FrameRef(6, 1)],
            interval: Duration::from_millis(100),
        },
        AnimState::Moving(AnimStateMoving::W) => Animation {
            frames: &[FrameRef(4, 2), FrameRef(4, 3)],
            interval: Duration::from_millis(100),
        },
        AnimState::Moving(AnimStateMoving::NW) => Animation {
            frames: &[FrameRef(1, 0), FrameRef(1, 1)],
            interval: Duration::from_millis(100),
        },
    }
}
