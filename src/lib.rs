use raw_window_handle as rwh_06;

pub struct WindowMain {
    pub hwnd: std::num::NonZero<isize>,
    pub hinstance: std::num::NonZero<isize>,
}

impl rwh_06::HasWindowHandle for WindowMain {
    fn window_handle(&self) -> Result<rwh_06::WindowHandle<'_>, rwh_06::HandleError> {
        unsafe {
            let mut handle = rwh_06::Win32WindowHandle::new(self.hwnd);
            handle.hinstance = Some(self.hinstance);
            Ok(rwh_06::WindowHandle::borrow_raw(
                rwh_06::RawWindowHandle::Win32(handle),
            ))
        }
    }
}

impl rwh_06::HasDisplayHandle for WindowMain {
    fn display_handle(&self) -> Result<rwh_06::DisplayHandle<'_>, rwh_06::HandleError> {
        unsafe {
            Ok(rwh_06::DisplayHandle::borrow_raw(
                rwh_06::RawDisplayHandle::Windows(rwh_06::WindowsDisplayHandle::new()),
            ))
        }
    }
}
