use windows::core::Result;

use windows::Win32::System::Com::{
    CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
};

/// Makes sure that COM lib is initialized long enought
pub fn init_com<T, F: FnOnce() -> T>(f: F) -> Result<T> {
    unsafe {
        CoInitializeEx(
            None,
            COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE,
        )?
    };

    let out = f();

    unsafe {
        CoUninitialize();
    }

    Ok(out)
}
