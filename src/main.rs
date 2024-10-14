#![windows_subsystem = "windows"]

use std::time::Instant;

use oneko_window::OnekoWindow;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{WindowId};

#[cfg(target_os = "linux")]
#[path = "native_utils_linux.rs"]
mod native_utils;
#[cfg(target_os = "windows")]
#[path = "native_utils_windows.rs"]
mod native_utils;

mod fill;
mod oneko;
mod oneko_window;
mod sprite_sheet;

fn main() -> Result<(), impl std::error::Error> {
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let event_loop = EventLoop::new().unwrap();

    let mut app = ControlFlowDemo::new();
    event_loop.run_app(&mut app)
}

struct ControlFlowDemo {
    oneko_window: Option<OnekoWindow>,
    next_update: Option<Instant>,
    wait_cancelled: bool,
    close_requested: bool,
}

impl ControlFlowDemo {
    fn new() -> Self {
        Self {
            oneko_window: None,
            next_update: None,
            wait_cancelled: false,
            close_requested: false,
        }
    }
}

impl ApplicationHandler for ControlFlowDemo {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        // info!("new_events: {cause:?}");

        match cause {
            StartCause::ResumeTimeReached { .. } | StartCause::Init => {
                if let Some(oneko_window) = &mut self.oneko_window {
                    oneko_window.update();
                    self.next_update = Some(oneko_window.next_update());
                } else {
                    self.next_update = None;
                }
            }
            _ => {}
        }

        self.wait_cancelled = match cause {
            StartCause::WaitCancelled { .. } => true,
            _ => false,
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let oneko_window = OnekoWindow::new(event_loop);
        self.next_update = Some(oneko_window.next_update());
        self.oneko_window = Some(oneko_window);
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(oneko_window) = &mut self.oneko_window {
            if window_id == oneko_window.window_id() {
                oneko_window.handle_window_event(&event);
            }
        }
        // info!("{event:?}");

        match event {
            WindowEvent::CloseRequested => {
                self.close_requested = true;
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if !self.wait_cancelled {
            if let Some(next_update) = self.next_update {
                event_loop.set_control_flow(ControlFlow::WaitUntil(next_update));
            }
        }

        if self.close_requested {
            event_loop.exit();
        }
    }
}
