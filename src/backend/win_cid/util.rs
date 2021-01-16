use winapi::{
    shared::winerror::{HRESULT, SUCCEEDED},
    um::{
        combaseapi::{CoInitializeEx, CoUninitialize},
        objbase::{COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE},
    },
};

use std::ptr;

pub trait ToResult {
    fn check(self) -> Result<HRESULT, HRESULT>;
}

impl ToResult for HRESULT {
    fn check(self) -> Result<HRESULT, HRESULT> {
        if SUCCEEDED(self) {
            Ok(self)
        } else {
            Err(self)
        }
    }
}

/// Makes sure that COM lib is initialized long enought
pub fn init_com<T, F: Fn() -> T>(f: F) -> Result<T, HRESULT> {
    unsafe {
        CoInitializeEx(
            ptr::null_mut(),
            COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE,
        )
        .check()?
    };

    let out = f();

    unsafe {
        CoUninitialize();
    }

    Ok(out)
}
