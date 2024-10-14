use std::rc::Rc;
use std::time::Instant;

use image::GenericImageView;
use rand::Rng;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, WindowEvent};
use winit::window::WindowId;
use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    window::{Window, WindowLevel},
};

#[cfg(target_os = "windows")]
use winit::platform::windows::WindowAttributesExtWindows;

#[cfg(target_os = "linux")]
use winit::platform::x11::{WindowAttributesExtX11, WindowType};

use crate::fill;
use crate::native_utils::get_cursor_position;
use crate::oneko::Oneko;

pub struct OnekoWindow {
    window: Rc<Window>,
    oneko: Oneko,
    next_update: Instant,
}

impl OnekoWindow {
    pub fn new(event_loop: &ActiveEventLoop) -> Self {
        let oneko = Oneko::default();

        let mut rng = rand::thread_rng();

        let monitor = event_loop
            .primary_monitor()
            .or_else(|| event_loop.available_monitors().next())
            .expect("Could not find any monitors");

        let monitor_position = monitor.position();
        let monitor_size = monitor.size();
        let (window_width, window_height) = oneko.get_frame().dimensions();
        let position = PhysicalPosition::new(
            rng.gen_range(50..(monitor_size.width - 50 - window_width) as i32) + monitor_position.x,
            rng.gen_range(50..(monitor_size.height - 50 - window_height) as i32)
                + monitor_position.y,
        );

        let window_attributes = Window::default_attributes()
            .with_title("oneko")
            .with_inner_size(PhysicalSize::new(32, 32))
            .with_resizable(false)
            .with_transparent(true)
            .with_decorations(false)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_position(position);

        #[cfg(target_os = "windows")]
        let window_attributes = window_attributes
            // .with_taskbar_icon(Some(oneko.get_icon(256)))
            .with_skip_taskbar(true);

        #[cfg(target_os = "linux")]
        let window_attributes = window_attributes.with_x11_window_type(vec![WindowType::Utility]);

        let window = event_loop.create_window(window_attributes).unwrap();
        window
            .set_cursor_hittest(false)
            .expect("Error disabling hit test");
        window.set_window_level(WindowLevel::AlwaysOnTop);

        Self {
            window: Rc::new(window),
            oneko,
            next_update: Instant::now(),
        }
    }

    pub fn update(&mut self) {
        let mut cursor_pos: (i32, i32) = get_cursor_position(&self.window)
            .expect("Error getting cursor position")
            .into();

        let mut window_position: (i32, i32) = self
            .window
            .outer_position()
            .expect("Error getting window position")
            .into();

        let monitor = self
            .window
            .current_monitor()
            .expect("Couldn't get current monitor");

        let monitor_position: (i32, i32) = monitor.position().into();
        let monitor_size: (i32, i32) = monitor.size().into();

        cursor_pos.0 -= monitor_position.0;
        cursor_pos.1 -= monitor_position.1;
        window_position.0 -= monitor_position.0;
        window_position.1 -= monitor_position.1;

        let (update_delay, mut new_window_position) =
            self.oneko.act(window_position, cursor_pos, monitor_size);
        self.next_update = Instant::now() + update_delay;

        self.window.request_redraw();

        new_window_position.0 += monitor_position.0;
        new_window_position.1 += monitor_position.1;

        if new_window_position != (0, 0) {
            self.window
                .set_outer_position(PhysicalPosition::<i32>::from(new_window_position));
        }
    }

    pub fn next_update(&self) -> Instant {
        self.next_update
    }

    pub fn window_id(&self) -> WindowId {
        self.window.id()
    }

    pub(crate) fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::MouseInput { state, .. } => {
                if *state == ElementState::Pressed {
                    self.oneko.click();
                    self.next_update = Instant::now();
                    self.window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                let data = self.oneko.get_frame();
                let current_size = self.window.inner_size();
                if current_size.width != data.width() || current_size.height != data.height() {
                    let _ = self
                        .window
                        .request_inner_size(PhysicalSize::new(data.width(), data.height()));
                }

                self.window.pre_present_notify();
                fill::fill_window(&self.window, data);
            }
            _ => (),
        }
    }
}
