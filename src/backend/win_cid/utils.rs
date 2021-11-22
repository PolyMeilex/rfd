use windows::core::Result;

use windows::Win32::System::Com::{
    CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
};

use std::ptr;

/// Makes sure that COM lib is initialized long enought
pub fn init_com<T, F: FnOnce() -> T>(f: F) -> Result<T> {
    unsafe {
        CoInitializeEx(
            ptr::null_mut(),
            COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE,
        )?
    };

    let out = f();

    unsafe {
        CoUninitialize();
    }

    Ok(out)
}
