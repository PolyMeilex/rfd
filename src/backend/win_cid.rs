//! Windows Common Item Dialog
//! Win32 Vista
use crate::DialogOptions;

use std::{path::PathBuf, ptr};

use winapi::{
    shared::{
        minwindef::{DWORD, LPVOID},
        ntdef::LPWSTR,
        winerror::HRESULT,
    },
    um::{
        combaseapi::CoTaskMemFree,
        shobjidl::{IFileOpenDialog, IFileSaveDialog, FOS_ALLOWMULTISELECT, FOS_PICKFOLDERS},
        shobjidl_core::{IShellItem, IShellItemArray, SIGDN_FILESYSPATH},
    },
};

mod utils {
    use crate::DialogOptions;

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
        pub fn build(params: &DialogOptions) -> Self {
            let mut filters = Vec::new();

            for f in params.filters.iter() {
                let name: Vec<u16> = OsStr::new(&f.name).encode_wide().chain(once(0)).collect();
                let mut ext_string = String::new();

                for e in f.extensions.iter() {
                    ext_string += &format!("*.{};", e);
                }

                let ext: Vec<u16> = OsStr::new(&ext_string)
                    .encode_wide()
                    .chain(once(0))
                    .collect();

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

    pub struct Dialog<T>(pub *mut T);

    impl<T> Dialog<T> {
        fn new_file_dialog(class: &GUID, id: &GUID) -> Result<*mut T, HRESULT> {
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

            Ok(dialog as *mut T)
        }
    }

    impl Dialog<IFileOpenDialog> {
        pub fn new() -> Result<Self, HRESULT> {
            let ptr = Self::new_file_dialog(&CLSID_FileOpenDialog, &IFileOpenDialog::uuidof())?;
            Ok(Self(ptr))
        }
    }

    impl Dialog<IFileSaveDialog> {
        pub fn new() -> Result<Self, HRESULT> {
            let ptr = Self::new_file_dialog(&CLSID_FileSaveDialog, &IFileSaveDialog::uuidof())?;
            Ok(Self(ptr))
        }
    }

    impl<T> Deref for Dialog<T> {
        type Target = *mut IFileDialog;
        fn deref(&self) -> &T {
            unsafe { &*self.0 }
        }
    }

    impl<T> Drop for Dialog<T> {
        fn drop(&mut self) {
            unsafe { (*(self.0 as *mut IFileDialog)).Release() };
        }
    }
}

use utils::*;

pub fn pick_file<'a>(params: impl Into<Option<DialogOptions<'a>>>) -> Option<PathBuf> {
    let params = params.into().unwrap_or_default();

    unsafe fn run(params: DialogOptions) -> Result<PathBuf, HRESULT> {
        init_com(|| {
            let dialog = Dialog::<IFileOpenDialog>::new()?;

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
        })?
    }

    unsafe { run(params).ok() }
}

pub fn save_file<'a>(params: impl Into<Option<DialogOptions<'a>>>) -> Option<PathBuf> {
    let params = params.into().unwrap_or_default();

    unsafe fn run(params: DialogOptions) -> Result<PathBuf, HRESULT> {
        init_com(|| {
            let dialog = Dialog::<IFileSaveDialog>::new()?;

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
        })?
    }

    unsafe { run(params).ok() }
}

pub fn pick_folder<'a>(params: impl Into<Option<DialogOptions<'a>>>) -> Option<PathBuf> {
    let params = params.into().unwrap_or_default();

    unsafe fn run() -> Result<PathBuf, HRESULT> {
        init_com(|| {
            let dialog = Dialog::<IFileOpenDialog>::new()?;

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
        })?
    }

    unsafe { run().ok() }
}

pub fn pick_files<'a>(params: impl Into<Option<DialogOptions<'a>>>) -> Option<Vec<PathBuf>> {
    let params = params.into().unwrap_or_default();

    unsafe fn run(params: DialogOptions) -> Result<Vec<PathBuf>, HRESULT> {
        init_com(|| {
            let dialog = Dialog::<IFileOpenDialog>::new()?;

            let flags: DWORD = FOS_ALLOWMULTISELECT;
            dialog.SetOptions(flags).check()?;

            let filters = Filters::build(&params);
            let spec = filters.as_spec();

            if !spec.is_empty() {
                dialog
                    .SetFileTypes(spec.len() as _, spec.as_ptr())
                    .check()?;
            }

            dialog.Show(ptr::null_mut()).check()?;

            let paths = {
                let mut res_items: *mut IShellItemArray = ptr::null_mut();
                dialog.GetResults(&mut res_items).check()?;

                let items = &*res_items;

                let mut count = 0;
                items.GetCount(&mut count);

                let mut paths = Vec::new();
                for id in 0..count {
                    let mut res_item: *mut IShellItem = ptr::null_mut();
                    items.GetItemAt(id, &mut res_item).check()?;

                    let mut display_name: LPWSTR = ptr::null_mut();

                    (*res_item)
                        .GetDisplayName(SIGDN_FILESYSPATH, &mut display_name)
                        .check()?;

                    let filename = to_os_string(&display_name);
                    CoTaskMemFree(display_name as LPVOID);

                    let path = PathBuf::from(filename);

                    paths.push(path);
                }

                items.Release();

                paths
            };

            Ok(paths)
        })?
    }

    unsafe { run(params).ok() }
}
