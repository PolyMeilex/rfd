//! Windows Common Item Dialog
//! Win32 Vista
use crate::DialogParams;
use std::ffi::OsString;
use std::os::windows::prelude::OsStringExt;
use std::{path::PathBuf, ptr};
use winapi::shared::winerror::HRESULT;
use winapi::um::combaseapi::CoTaskMemFree;
use winapi::um::shobjidl_core::SIGDN_FILESYSPATH;

use winapi::shared::minwindef::LPVOID;
use winapi::shared::ntdef::LPWSTR;
use winapi::shared::winerror::SUCCEEDED;
use winapi::shared::wtypesbase::CLSCTX_INPROC_SERVER;
use winapi::um::shobjidl_core::IShellItem;

use winapi::{
    um::{
        combaseapi::{CoCreateInstance, CoInitializeEx},
        objbase::{COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE},
        shobjidl::{IFileDialog, IFileOpenDialog},
        shobjidl_core::CLSID_FileOpenDialog,
    },
    Interface,
};

trait ToResoult {
    fn check(self) -> Result<(), HRESULT>;
}

impl ToResoult for HRESULT {
    fn check(self) -> Result<(), HRESULT> {
        if SUCCEEDED(self) {
            Ok(())
        } else {
            Err(self)
        }
    }
}

unsafe fn init() -> Result<(), HRESULT> {
    CoInitializeEx(
        ptr::null_mut(),
        COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE,
    )
    .check()
}

fn to_os_string(s: &LPWSTR) -> OsString {
    let slice = unsafe {
        let mut len = 0;
        while *s.offset(len) != 0 {
            len += 1;
        }
        std::slice::from_raw_parts(*s, len as usize)
    };
    OsStringExt::from_wide(slice)
}

pub fn open_file_with_params(params: DialogParams) -> Option<PathBuf> {
    unsafe fn run(params: DialogParams) -> Result<PathBuf, HRESULT> {
        init()?;
        let mut dialog: *mut IFileDialog = ptr::null_mut();

        let class = &CLSID_FileOpenDialog;
        let id = IFileOpenDialog::uuidof();

        CoCreateInstance(
            class,
            ptr::null_mut(),
            CLSCTX_INPROC_SERVER,
            &id,
            &mut dialog as *mut *mut IFileDialog as *mut LPVOID,
        )
        .check()?;

        (*dialog).Show(ptr::null_mut()).check()?;

        let mut res_item: *mut IShellItem = ptr::null_mut();
        (*dialog).GetResult(&mut res_item).check()?;

        let mut display_name: LPWSTR = ptr::null_mut();

        (*res_item)
            .GetDisplayName(SIGDN_FILESYSPATH, &mut display_name)
            .check()?;

        let filename = to_os_string(&display_name);
        CoTaskMemFree(display_name as LPVOID);

        Ok(PathBuf::from(filename))
    }

    unsafe { run(params).ok() }
}

pub fn save_file_with_params(params: DialogParams) -> Option<PathBuf> {
    unimplemented!("pick_folder");
}

pub fn pick_folder() -> Option<PathBuf> {
    unimplemented!("pick_folder");
}

pub fn open_multiple_files_with_params(params: DialogParams) -> Option<Vec<PathBuf>> {
    unimplemented!("open_multiple_files_with_params");
}
