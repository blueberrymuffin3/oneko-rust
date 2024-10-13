use std::cell::RefCell;

use tracing::info;
use winit::{
    dpi::PhysicalPosition,
    raw_window_handle::{HandleError, HasDisplayHandle, RawDisplayHandle},
};
use x11rb::{
    connection::Connection,
    protocol::xproto::{self, QueryPointerReply},
    xcb_ffi::XCBConnection,
};

thread_local! {
  static XCB_CONNECTION: RefCell<Option<x11rb::xcb_ffi::XCBConnection>> = RefCell::new(None);
}

pub fn with_x11_connection<R>(body: impl FnOnce(&mut XCBConnection) -> R) -> R {
    XCB_CONNECTION.with(|maybe_connection| {
        let mut binding = maybe_connection.borrow_mut();
        let connection = binding.get_or_insert_with(|| {
            XCBConnection::connect(None)
                .expect("Error connecting to X11")
                .0
        });
        body(connection)
    })
}

pub fn get_cursor_position(
    handle: impl HasDisplayHandle,
) -> Result<PhysicalPosition<i32>, HandleError> {
    let display_handle = handle.display_handle()?.as_raw();

    match display_handle {
        RawDisplayHandle::Xlib(xlib_display) => {
            xlib_display.display.expect("No xlib display");

            with_x11_connection(|conn| -> Result<PhysicalPosition<i32>, HandleError> {
                let setup = conn.setup();

                let mut found_pointer: Option<QueryPointerReply> = None;
                for screen in &setup.roots {
                    let window: xproto::Window = screen.root;

                    let Ok(pointer_query_cookie) = xproto::query_pointer(conn, window) else {
                        continue;
                    };

                    let Ok(pointer_query) = pointer_query_cookie.reply() else {
                        continue;
                    };

                    found_pointer = Some(pointer_query);
                    break;
                }

                let Some(pointer_query) = found_pointer else {
                    panic!("Pointer could not be found");
                };

                Ok(PhysicalPosition::new(
                    pointer_query.root_x.into(),
                    pointer_query.root_y.into(),
                ))
                // Ok(PhysicalPosition::new(pointer_query.win_x, pointer_query.win_y))
            })
        }
        // _ => Err(HandleError::NotSupported),
        _ => Ok(PhysicalPosition::new(400, 400)),
    }
}
