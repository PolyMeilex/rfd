use std::fmt;

mod ffi;
mod raw;

#[derive(Debug)]
pub struct WaylandWindowIdentifier {
    handle: raw::XdgForeignHandle,
}

impl WaylandWindowIdentifier {
    pub unsafe fn from_raw(
        surface_ptr: *mut std::ffi::c_void,
        display_ptr: *mut std::ffi::c_void,
    ) -> Option<Self> {
        if surface_ptr.is_null() || display_ptr.is_null() {
            return None;
        }

        Some(Self {
            handle: raw::run(display_ptr, surface_ptr)?,
        })
    }
}

impl fmt::Display for WaylandWindowIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.handle.fmt(f)
    }
}
