//! Windows Common Item Dialog
//! Win32 Vista
use crate::FileDialog;

use std::{
    ffi::{OsStr, OsString},
    iter::once,
    ops::Deref,
    os::windows::{ffi::OsStrExt, prelude::OsStringExt},
    path::{Path, PathBuf},
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
        combaseapi::{CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize},
        objbase::{COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE},
        shobjidl::{
            IFileDialog, IFileOpenDialog, IFileSaveDialog, FOS_ALLOWMULTISELECT, FOS_PICKFOLDERS,
        },
        shobjidl_core::{
            CLSID_FileOpenDialog, CLSID_FileSaveDialog, IShellItem, IShellItemArray,
            SHCreateItemFromParsingName, SIGDN_FILESYSPATH,
        },
        shtypes::COMDLG_FILTERSPEC,
    },
    Interface,
};

pub fn pick_file<'a>(opt: &FileDialog<'a>) -> Option<PathBuf> {
    unsafe fn run<'a>(opt: &FileDialog<'a>) -> Result<PathBuf, HRESULT> {
        init_com(|| {
            let dialog = Dialog::new_open_dialog()?;

            dialog.add_filters(&opt.filters)?;
            dialog.set_path(&opt.starting_directory)?;

            dialog.Show(ptr::null_mut()).check()?;

            dialog.get_result()
        })?
    }

    unsafe { run(opt).ok() }
}

pub fn save_file<'a>(opt: &FileDialog<'a>) -> Option<PathBuf> {
    unsafe fn run<'a>(opt: &FileDialog<'a>) -> Result<PathBuf, HRESULT> {
        init_com(|| {
            let dialog = Dialog::new_save_dialog()?;

            dialog.add_filters(&opt.filters)?;
            dialog.set_path(&opt.starting_directory)?;

            dialog.Show(ptr::null_mut()).check()?;

            dialog.get_result()
        })?
    }

    unsafe { run(opt).ok() }
}

pub fn pick_folder<'a>(opt: &FileDialog<'a>) -> Option<PathBuf> {
    unsafe fn run<'a>(opt: &FileDialog<'a>) -> Result<PathBuf, HRESULT> {
        init_com(|| {
            let dialog = Dialog::new_open_dialog()?;

            dialog.set_path(&opt.starting_directory)?;

            dialog.SetOptions(FOS_PICKFOLDERS).check()?;

            dialog.Show(ptr::null_mut()).check()?;

            dialog.get_result()
        })?
    }

    unsafe { run(opt).ok() }
}

pub fn pick_files<'a>(opt: &FileDialog<'a>) -> Option<Vec<PathBuf>> {
    unsafe fn run<'a>(opt: &FileDialog<'a>) -> Result<Vec<PathBuf>, HRESULT> {
        init_com(|| {
            let dialog = Dialog::new_open_dialog()?;

            dialog.add_filters(&opt.filters)?;
            dialog.set_path(&opt.starting_directory)?;

            dialog.SetOptions(FOS_ALLOWMULTISELECT).check()?;

            dialog.Show(ptr::null_mut()).check()?;

            dialog.get_results()
        })?
    }

    unsafe { run(opt).ok() }
}

//
//
//

use std::future::Future;

pub fn pick_file_async<'a>(opt: &FileDialog<'a>) -> Future<Output = Option<FileHandle>> {
    unimplemented!("");
}

pub fn save_file_async<'a>(opt: &FileDialog<'a>) -> Future<Output = Option<FileHandle>> {
    unimplemented!("");
}

pub fn pick_folder_async<'a>(opt: &FileDialog<'a>) -> Future<Output = Option<FileHandle>> {
    unimplemented!("");
}

pub fn pick_files_async<'a>(opt: &FileDialog<'a>) -> Future<Output = Option<Vec<FileHandle>>> {
    unimplemented!("");
}

//
// Internal
//

trait ToResult {
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
fn init_com<T, F: Fn() -> T>(f: F) -> Result<T, HRESULT> {
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

struct Dialog(pub *mut IFileDialog);

impl Dialog {
    fn new_file_dialog(class: &GUID, id: &GUID) -> Result<*mut IFileDialog, HRESULT> {
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

        Ok(dialog)
    }

    fn new_open_dialog() -> Result<Self, HRESULT> {
        let ptr = Self::new_file_dialog(&CLSID_FileOpenDialog, &IFileOpenDialog::uuidof())?;
        Ok(Self(ptr))
    }

    fn new_save_dialog() -> Result<Self, HRESULT> {
        let ptr = Self::new_file_dialog(&CLSID_FileSaveDialog, &IFileSaveDialog::uuidof())?;
        Ok(Self(ptr))
    }

    fn add_filters(&self, filters: &[crate::Filter]) -> Result<(), HRESULT> {
        let f_list = {
            let mut f_list = Vec::new();

            for f in filters.iter() {
                let name: Vec<u16> = OsStr::new(&f.name).encode_wide().chain(once(0)).collect();
                let mut ext_string = String::new();

                for e in f.extensions.iter() {
                    ext_string += &format!("*.{};", e);
                }

                let ext: Vec<u16> = OsStr::new(&ext_string)
                    .encode_wide()
                    .chain(once(0))
                    .collect();

                f_list.push((name, ext));
            }
            f_list
        };

        let spec: Vec<_> = f_list
            .iter()
            .map(|(name, ext)| COMDLG_FILTERSPEC {
                pszName: name.as_ptr(),
                pszSpec: ext.as_ptr(),
            })
            .collect();

        unsafe {
            if !spec.is_empty() {
                (*self.0)
                    .SetFileTypes(spec.len() as _, spec.as_ptr())
                    .check()?;
            }
        }
        Ok(())
    }

    pub fn set_path(&self, path: &Option<&Path>) -> Result<(), HRESULT> {
        if let Some(path) = path {
            if let Some(path) = path.to_str() {
                let wide_path: Vec<u16> = OsStr::new(path).encode_wide().chain(once(0)).collect();

                unsafe {
                    let mut item: *mut IShellItem = ptr::null_mut();
                    SHCreateItemFromParsingName(
                        wide_path.as_ptr(),
                        ptr::null_mut(),
                        &IShellItem::uuidof(),
                        &mut item as *mut *mut IShellItem as *mut *mut _,
                    )
                    .check()?;

                    (*self.0).SetDefaultFolder(item).check()?;
                }
            }
        }
        Ok(())
    }

    fn get_results(&self) -> Result<Vec<PathBuf>, HRESULT> {
        unsafe {
            let mut res_items: *mut IShellItemArray = ptr::null_mut();
            (*(self.0 as *mut IFileOpenDialog))
                .GetResults(&mut res_items)
                .check()?;

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

            Ok(paths)
        }
    }

    fn get_result(&self) -> Result<PathBuf, HRESULT> {
        let mut res_item: *mut IShellItem = ptr::null_mut();
        unsafe {
            (*self.0).GetResult(&mut res_item).check()?;

            let mut display_name: LPWSTR = ptr::null_mut();

            (*res_item)
                .GetDisplayName(SIGDN_FILESYSPATH, &mut display_name)
                .check()?;

            let filename = to_os_string(&display_name);
            CoTaskMemFree(display_name as LPVOID);

            Ok(PathBuf::from(filename))
        }
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
        unsafe { (*(self.0 as *mut IFileDialog)).Release() };
    }
}
