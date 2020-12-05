//! Windows Common Item Dialog
//! Win32 Vista
use crate::DialogParams;

use winapi::shared::minwindef::DWORD;
use winapi::um::shobjidl::FOS_ALLOWMULTISELECT;
use winapi::um::shobjidl::FOS_PICKFOLDERS;

use std::{path::PathBuf, ptr};

use winapi::{
    shared::{minwindef::LPVOID, ntdef::LPWSTR, winerror::HRESULT},
    um::{combaseapi::CoTaskMemFree, shobjidl_core::IShellItem, shobjidl_core::SIGDN_FILESYSPATH},
};

mod utils {
    use crate::DialogParams;

    use std::{
        ffi::{OsStr, OsString},
        iter::once,
        ops::Deref,
        os::windows::{ffi::OsStrExt, prelude::OsStringExt},
        ptr,
    };

    use winapi::{
        shared::{
            guiddef::GUID,
            minwindef::LPVOID,
            ntdef::LPWSTR,
            winerror::{HRESULT, SUCCEEDED},
            wtypesbase::CLSCTX_INPROC_SERVER,
        },
        um::{
            combaseapi::{CoCreateInstance, CoInitializeEx, CoUninitialize},
            objbase::{COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE},
            shobjidl::{IFileDialog, IFileOpenDialog, IFileSaveDialog},
            shobjidl_core::{CLSID_FileOpenDialog, CLSID_FileSaveDialog},
            shtypes::COMDLG_FILTERSPEC,
        },
        Interface,
    };

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

    pub fn to_os_string(s: &LPWSTR) -> OsString {
        let slice = unsafe {
            let mut len = 0;
            while *s.offset(len) != 0 {
                len += 1;
            }
            std::slice::from_raw_parts(*s, len as usize)
        };
        OsStringExt::from_wide(slice)
    }

    pub struct Filters(Vec<(Vec<u16>, Vec<u16>)>);

    impl Filters {
        pub fn build(params: &DialogParams) -> Self {
            let mut filters = Vec::new();

            for f in params.filters.iter() {
                let name: Vec<u16> = OsStr::new(&f.0).encode_wide().chain(once(0)).collect();
                let ext: Vec<u16> = OsStr::new(&f.1).encode_wide().chain(once(0)).collect();

                filters.push((name, ext));
            }

            Self(filters)
        }
        pub fn as_spec(&self) -> Vec<COMDLG_FILTERSPEC> {
            self.0
                .iter()
                .map(|(name, ext)| COMDLG_FILTERSPEC {
                    pszName: name.as_ptr(),
                    pszSpec: ext.as_ptr(),
                })
                .collect()
        }
    }

    pub struct Dialog(*mut IFileDialog);

    impl Dialog {
        pub fn new(class: &GUID, id: &GUID) -> Result<Self, HRESULT> {
            let mut dialog: *mut IFileDialog = ptr::null_mut();

            unsafe {
                CoCreateInstance(
                    class,
                    ptr::null_mut(),
                    CLSCTX_INPROC_SERVER,
                    id,
                    &mut dialog as *mut *mut IFileDialog as *mut LPVOID,
                )
                .check()?;
            }

            Ok(Self(dialog))
        }
        pub fn new_open_dialog() -> Result<Self, HRESULT> {
            Self::new(&CLSID_FileOpenDialog, &IFileOpenDialog::uuidof())
        }
        pub fn new_save_dialog() -> Result<Self, HRESULT> {
            Self::new(&CLSID_FileSaveDialog, &IFileSaveDialog::uuidof())
        }
    }

    impl Deref for Dialog {
        type Target = IFileDialog;
        fn deref(&self) -> &Self::Target {
            unsafe { &*self.0 }
        }
    }

    impl Drop for Dialog {
        fn drop(&mut self) {
            unsafe { (*self.0).Release() };
        }
    }
}

use utils::*;

pub fn open_file_with_params(params: DialogParams) -> Option<PathBuf> {
    unsafe fn run(params: DialogParams) -> Result<PathBuf, HRESULT> {
        let res = init_com(|| {
            let dialog = Dialog::new_open_dialog()?;

            let filters = Filters::build(&params);
            let spec = filters.as_spec();

            if !spec.is_empty() {
                dialog
                    .SetFileTypes(spec.len() as _, spec.as_ptr())
                    .check()?;
            }

            dialog.Show(ptr::null_mut()).check()?;

            let mut res_item: *mut IShellItem = ptr::null_mut();
            dialog.GetResult(&mut res_item).check()?;

            let mut display_name: LPWSTR = ptr::null_mut();

            (*res_item)
                .GetDisplayName(SIGDN_FILESYSPATH, &mut display_name)
                .check()?;

            let filename = to_os_string(&display_name);
            CoTaskMemFree(display_name as LPVOID);

            Ok(PathBuf::from(filename))
        })?;

        res
    }

    unsafe { run(params).ok() }
}

pub fn save_file_with_params(params: DialogParams) -> Option<PathBuf> {
    unsafe fn run(params: DialogParams) -> Result<PathBuf, HRESULT> {
        let res = init_com(|| {
            let dialog = Dialog::new_save_dialog()?;

            let filters = Filters::build(&params);
            let spec = filters.as_spec();

            if !spec.is_empty() {
                dialog
                    .SetFileTypes(spec.len() as _, spec.as_ptr())
                    .check()?;
            }

            dialog.Show(ptr::null_mut()).check()?;

            let mut res_item: *mut IShellItem = ptr::null_mut();
            dialog.GetResult(&mut res_item).check()?;

            let mut display_name: LPWSTR = ptr::null_mut();

            (*res_item)
                .GetDisplayName(SIGDN_FILESYSPATH, &mut display_name)
                .check()?;

            let filename = to_os_string(&display_name);
            CoTaskMemFree(display_name as LPVOID);

            Ok(PathBuf::from(filename))
        })?;

        res
    }

    unsafe { run(params).ok() }
}

pub fn pick_folder() -> Option<PathBuf> {
    unsafe fn run() -> Result<PathBuf, HRESULT> {
        let res = init_com(|| {
            let dialog = Dialog::new_open_dialog()?;

            let flags: DWORD = FOS_PICKFOLDERS;

            dialog.SetOptions(flags).check()?;

            dialog.Show(ptr::null_mut()).check()?;

            let mut res_item: *mut IShellItem = ptr::null_mut();
            dialog.GetResult(&mut res_item).check()?;

            let mut display_name: LPWSTR = ptr::null_mut();

            (*res_item)
                .GetDisplayName(SIGDN_FILESYSPATH, &mut display_name)
                .check()?;

            let filename = to_os_string(&display_name);
            CoTaskMemFree(display_name as LPVOID);

            Ok(PathBuf::from(filename))
        })?;

        res
    }

    unsafe { run().ok() }
}

pub fn open_multiple_files_with_params(params: DialogParams) -> Option<Vec<PathBuf>> {
    unimplemented!("open_multiple_files_with_params");
}
