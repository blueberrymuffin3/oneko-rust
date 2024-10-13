use windows::Win32::Foundation::POINT;
use winit::{dpi::PhysicalPosition, raw_window_handle::{HandleError, HasDisplayHandle}};

pub fn get_cursor_position(
    _handle: impl HasDisplayHandle,
) -> Result<PhysicalPosition<i32>, HandleError> {
    unsafe {
        let mut point: POINT = Default::default();
        windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut point as *mut POINT).expect("Error getting cursor position");

        Ok(PhysicalPosition::new(point.x, point.y))
    }
}
