use crate::FileDialog;

use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt, path::PathBuf};

use windows::core::Result;
use windows::Win32::{
    Foundation::{HWND, PWSTR},
    System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_INPROC_SERVER},
    UI::Shell::{
        Common::COMDLG_FILTERSPEC, FileOpenDialog, FileSaveDialog, IFileDialog, IFileOpenDialog,
        IFileSaveDialog, IShellItem, SHCreateItemFromParsingName, FOS_ALLOWMULTISELECT,
        FOS_PICKFOLDERS, SIGDN_FILESYSPATH,
    },
};

#[cfg(feature = "parent")]
use raw_window_handle::RawWindowHandle;

unsafe fn read_to_string(ptr: PWSTR) -> String {
    let mut len = 0usize;
    let mut cursor = ptr;
    loop {
        let val = cursor.0.read();
        if val == 0 {
            break;
        }
        len += 1;
        cursor = PWSTR(cursor.0.add(1));
    }

    let slice = std::slice::from_raw_parts(ptr.0, len);
    String::from_utf16(slice).unwrap()
}

pub enum DialogKind {
    Open(IFileOpenDialog),
    Save(IFileSaveDialog),
}

impl DialogKind {
    fn as_dialog(&self) -> IFileDialog {
        match self {
            Self::Open(d) => d.into(),
            Self::Save(d) => d.into(),
        }
    }
}

pub struct IDialog(pub DialogKind, Option<HWND>);

impl IDialog {
    fn new_open_dialog(opt: &FileDialog) -> Result<Self> {
        let dialog: IFileOpenDialog =
            unsafe { CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)? };

        #[cfg(feature = "parent")]
        let parent = match opt.parent {
            Some(RawWindowHandle::Win32(handle)) => Some(HWND(handle.hwnd as _)),
            None => None,
            _ => unreachable!("unsupported window handle, expected: Windows"),
        };
        #[cfg(not(feature = "parent"))]
        let parent = None;

        Ok(Self(DialogKind::Open(dialog), parent))
    }

    fn new_save_dialog(opt: &FileDialog) -> Result<Self> {
        let dialog: IFileSaveDialog =
            unsafe { CoCreateInstance(&FileSaveDialog, None, CLSCTX_INPROC_SERVER)? };

        #[cfg(feature = "parent")]
        let parent = match opt.parent {
            Some(RawWindowHandle::Win32(handle)) => Some(HWND(handle.hwnd as _)),
            None => None,
            _ => unreachable!("unsupported window handle, expected: Windows"),
        };
        #[cfg(not(feature = "parent"))]
        let parent = None;

        Ok(Self(DialogKind::Save(dialog), parent))
    }

    fn add_filters(&self, filters: &[crate::file_dialog::Filter]) -> Result<()> {
        if let Some(first_filter) = filters.first() {
            if let Some(first_extension) = first_filter.extensions.first() {
                let mut extension: Vec<u16> =
                    first_extension.encode_utf16().chain(Some(0)).collect();
                unsafe {
                    self.0
                        .as_dialog()
                        .SetDefaultExtension(PWSTR(extension.as_mut_ptr()))?;
                }
            }
        }

        let mut f_list = {
            let mut f_list = Vec::new();

            for f in filters.iter() {
                let name: Vec<u16> = OsStr::new(&f.name).encode_wide().chain(once(0)).collect();
                let ext_string = f
                    .extensions
                    .iter()
                    .map(|item| format!("*.{}", item))
                    .collect::<Vec<_>>()
                    .join(";");

                let ext: Vec<u16> = OsStr::new(&ext_string)
                    .encode_wide()
                    .chain(once(0))
                    .collect();

                f_list.push((name, ext));
            }
            f_list
        };

        let spec: Vec<_> = f_list
            .iter_mut()
            .map(|(name, ext)| COMDLG_FILTERSPEC {
                pszName: PWSTR(name.as_mut_ptr()),
                pszSpec: PWSTR(ext.as_mut_ptr()),
            })
            .collect();

        unsafe {
            if !spec.is_empty() {
                self.0
                    .as_dialog()
                    .SetFileTypes(spec.len() as _, spec.as_ptr())?;
            }
        }
        Ok(())
    }

    fn set_path(&self, path: &Option<PathBuf>) -> Result<()> {
        if let Some(path) = path {
            if let Some(path) = path.to_str() {
                // Strip Win32 namespace prefix from the path
                let path = path.strip_prefix(r"\\?\").unwrap_or(path);

                let mut wide_path: Vec<u16> =
                    OsStr::new(path).encode_wide().chain(once(0)).collect();

                unsafe {
                    let item: Option<IShellItem> =
                        SHCreateItemFromParsingName(PWSTR(wide_path.as_mut_ptr()), None).ok();

                    if let Some(item) = item {
                        // For some reason SetDefaultFolder(), does not guarantees default path, so we use SetFolder
                        self.0.as_dialog().SetFolder(item)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn set_file_name(&self, file_name: &Option<String>) -> Result<()> {
        if let Some(path) = file_name {
            let mut wide_path: Vec<u16> = OsStr::new(path).encode_wide().chain(once(0)).collect();

            unsafe {
                self.0
                    .as_dialog()
                    .SetFileName(PWSTR(wide_path.as_mut_ptr()))?;
            }
        }
        Ok(())
    }

    fn set_title(&self, title: &Option<String>) -> Result<()> {
        if let Some(title) = title {
            let mut wide_title: Vec<u16> = OsStr::new(title).encode_wide().chain(once(0)).collect();

            unsafe {
                self.0
                    .as_dialog()
                    .SetTitle(PWSTR(wide_title.as_mut_ptr()))?;
            }
        }
        Ok(())
    }

    pub fn get_results(&self) -> Result<Vec<PathBuf>> {
        unsafe {
            let dialog = if let DialogKind::Open(ref d) = self.0 {
                d
            } else {
                unreachable!()
            };

            let items = dialog.GetResults()?;

            let count = items.GetCount()?;

            let mut paths = Vec::new();
            for id in 0..count {
                let res_item = items.GetItemAt(id)?;

                let display_name = res_item.GetDisplayName(SIGDN_FILESYSPATH)?;

                let filename = read_to_string(display_name);

                CoTaskMemFree(display_name.0 as _);

                let path = PathBuf::from(filename);
                paths.push(path);
            }

            Ok(paths)
        }
    }

    pub fn get_result(&self) -> Result<PathBuf> {
        unsafe {
            let res_item = self.0.as_dialog().GetResult()?;
            let display_name = res_item.GetDisplayName(SIGDN_FILESYSPATH)?;

            let filename = read_to_string(display_name);
            CoTaskMemFree(display_name.0 as _);

            Ok(PathBuf::from(filename))
        }
    }

    pub fn show(&self) -> Result<()> {
        unsafe { self.0.as_dialog().Show(self.1) }
    }
}

impl IDialog {
    pub fn build_pick_file(opt: &FileDialog) -> Result<Self> {
        let dialog = IDialog::new_open_dialog(opt)?;

        dialog.add_filters(&opt.filters)?;
        dialog.set_path(&opt.starting_directory)?;
        dialog.set_file_name(&opt.file_name)?;
        dialog.set_title(&opt.title)?;

        Ok(dialog)
    }

    pub fn build_save_file(opt: &FileDialog) -> Result<Self> {
        let dialog = IDialog::new_save_dialog(opt)?;

        dialog.add_filters(&opt.filters)?;
        dialog.set_path(&opt.starting_directory)?;
        dialog.set_file_name(&opt.file_name)?;
        dialog.set_title(&opt.title)?;

        Ok(dialog)
    }

    pub fn build_pick_folder(opt: &FileDialog) -> Result<Self> {
        let dialog = IDialog::new_open_dialog(opt)?;

        dialog.set_path(&opt.starting_directory)?;
        dialog.set_title(&opt.title)?;

        unsafe {
            dialog.0.as_dialog().SetOptions(FOS_PICKFOLDERS as _)?;
        }

        Ok(dialog)
    }

    pub fn build_pick_files(opt: &FileDialog) -> Result<Self> {
        let dialog = IDialog::new_open_dialog(opt)?;

        dialog.add_filters(&opt.filters)?;
        dialog.set_path(&opt.starting_directory)?;
        dialog.set_file_name(&opt.file_name)?;
        dialog.set_title(&opt.title)?;

        unsafe {
            dialog.0.as_dialog().SetOptions(FOS_ALLOWMULTISELECT as _)?;
        }

        Ok(dialog)
    }
}
