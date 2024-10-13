//! Fill the window buffer with a solid color.
//!
//! Launching a window without drawing to it has unpredictable results varying from platform to
//! platform. In order to have well-defined examples, this module provides an easy way to
//! fill the window buffer with a solid color.
//!
//! The `softbuffer` crate is used, largely because of its ease of use. `glutin` or `wgpu` could
//! also be used to fill the window buffer, but they are more complicated to use.

use std::mem::ManuallyDrop;
use std::num::NonZeroU32;
#[allow(unused_imports)]
use std::{cell::RefCell, rc::Rc};
use std::{cmp::min, collections::HashMap};

use image::{GenericImageView, RgbaImage, SubImage};
use softbuffer::{Context, Surface};
use winit::window::{Window, WindowId};

thread_local! {
    // NOTE: You should never do things like that, create context and drop it before
    // you drop the event loop. We do this for brevity to not blow up examples. We use
    // ManuallyDrop to prevent destructors from running.
    //
    // A static, thread-local map of graphics contexts to open windows.
    static GC: ManuallyDrop<RefCell<Option<GraphicsContext>>> = const { ManuallyDrop::new(RefCell::new(None)) };
}

/// The graphics context used to draw to a window.
pub struct GraphicsContext {
    /// The global softbuffer context.
    context: RefCell<Context<Rc<Window>>>,

    /// The hash map of window IDs to surfaces.
    surfaces: HashMap<WindowId, Surface<Rc<Window>, Rc<Window>>>,
}

impl GraphicsContext {
    fn new(w: Rc<Window>) -> Self {
        Self {
            context: RefCell::new(Context::new(w).expect("Failed to create a softbuffer context")),
            surfaces: HashMap::new(),
        }
    }

    fn create_surface(&mut self, window: &Rc<Window>) -> &mut Surface<Rc<Window>, Rc<Window>> {
        self.surfaces.entry(window.id()).or_insert_with(|| {
            Surface::new(&self.context.borrow(), window.clone())
                .expect("Failed to create a softbuffer surface")
        })
    }

    fn destroy_surface(&mut self, window: &Window) {
        self.surfaces.remove(&window.id());
    }
}

pub fn fill_window(window: &Rc<Window>, data: SubImage<&RgbaImage>) {
    GC.with(|gc| {
        let size = window.inner_size();
        let (Some(width), Some(height)) =
            (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
        else {
            return;
        };

        // Either get the last context used or create a new one.
        let mut gc = gc.borrow_mut();
        let surface = gc
            .get_or_insert_with(|| GraphicsContext::new(window.clone()))
            .create_surface(window);

        const TRANSPARENCY: u32 = 0x00000000;

        surface
            .resize(width, height)
            .expect("Failed to resize the softbuffer surface");

        let mut buffer = surface
            .buffer_mut()
            .expect("Failed to get the softbuffer buffer");

        buffer.fill(TRANSPARENCY);
        for y in 0..min(size.height, data.height()) {
            for x in 0..min(size.width, data.width()) {
                let image::Rgba([r, g, b, a]) = data.get_pixel(x, y);
                let pixel_data: u32 =
                    (a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | (b as u32);
                buffer[(x + y * size.width) as usize] = pixel_data;
            }
        }

        buffer
            .present()
            .expect("Failed to present the softbuffer buffer");
    })
}

#[allow(dead_code)]
pub fn cleanup_window(window: &Window) {
    GC.with(|gc| {
        let mut gc = gc.borrow_mut();
        if let Some(context) = gc.as_mut() {
            context.destroy_surface(window);
        }
    });
}
