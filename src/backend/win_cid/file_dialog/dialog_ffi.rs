use super::super::utils::str_to_vec_u16;
use crate::FileDialog;

use std::{ffi::c_void, path::PathBuf};

use windows_sys::core::{GUID, HRESULT, PCWSTR, PWSTR};
use windows_sys::Win32::{
    Foundation::HWND,
    System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_INPROC_SERVER},
    UI::Shell::{
        Common::COMDLG_FILTERSPEC, FileOpenDialog, FileSaveDialog, SHCreateItemFromParsingName,
        FILEOPENDIALOGOPTIONS, FOS_ALLOWMULTISELECT, FOS_PICKFOLDERS, SIGDN, SIGDN_FILESYSPATH,
    },
};

use raw_window_handle::RawWindowHandle;

#[inline]
unsafe fn read_to_string(ptr: *const u16) -> String {
    let mut cursor = ptr;

    loop {
        if *cursor == 0 {
            break;
        }

        cursor = cursor.add(1);
    }

    let slice = std::slice::from_raw_parts(ptr, cursor.offset_from(ptr) as usize);
    String::from_utf16(slice).unwrap()
}

pub type Result<T> = std::result::Result<T, HRESULT>;

#[inline]
fn wrap_err(hresult: HRESULT) -> Result<()> {
    if hresult >= 0 {
        Ok(())
    } else {
        Err(hresult)
    }
}

#[repr(C)]
struct Interface<T> {
    vtable: *mut T,
}

impl<T> Interface<T> {
    #[inline]
    fn vtbl(&self) -> &T {
        unsafe { &*self.vtable }
    }
}

#[repr(C)]
struct IUnknownV {
    __query_interface: usize,
    __add_ref: usize,
    release: unsafe extern "system" fn(this: *mut c_void) -> u32,
}

type IUnknown = Interface<IUnknownV>;

#[inline]
fn drop_impl(ptr: *mut std::ffi::c_void) {
    unsafe {
        ((*ptr.cast::<IUnknown>()).vtbl().release)(ptr);
    }
}

#[repr(C)]
struct IShellItemV {
    base: IUnknownV,
    __bind_to_handler: usize,
    __get_parent: usize,
    get_display_name:
        unsafe extern "system" fn(this: *mut c_void, name_look: SIGDN, name: *mut PWSTR) -> HRESULT,
    __get_attributes: usize,
    __compare: usize,
}

#[repr(C)]
struct IShellItem(*mut Interface<IShellItemV>);

impl IShellItem {
    fn get_path(&self) -> Result<PathBuf> {
        let filename = unsafe {
            let mut dname = std::mem::MaybeUninit::uninit();
            wrap_err(((*self.0).vtbl().get_display_name)(
                self.0.cast(),
                SIGDN_FILESYSPATH,
                dname.as_mut_ptr(),
            ))?;

            let dname = dname.assume_init();
            let fname = read_to_string(dname);
            CoTaskMemFree(dname.cast());
            fname
        };

        Ok(filename.into())
    }
}

impl Drop for IShellItem {
    fn drop(&mut self) {
        drop_impl(self.0.cast());
    }
}

#[repr(C)]
struct IShellItemArrayV {
    base: IUnknownV,
    __bind_to_handler: usize,
    __get_property_store: usize,
    __get_property_description_list: usize,
    __get_attributes: usize,
    get_count: unsafe extern "system" fn(this: *mut c_void, num_items: *mut u32) -> HRESULT,
    get_item_at: unsafe extern "system" fn(
        this: *mut c_void,
        dwindex: u32,
        ppsi: *mut IShellItem,
    ) -> HRESULT,
    __enum_items: usize,
}

#[repr(C)]
struct IShellItemArray(*mut Interface<IShellItemArrayV>);

impl Drop for IShellItemArray {
    fn drop(&mut self) {
        drop_impl(self.0.cast());
    }
}

#[repr(C)]
struct IModalWindowV {
    base: IUnknownV,
    show: unsafe extern "system" fn(this: *mut c_void, owner: HWND) -> HRESULT,
}

/// <https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-ifiledialog>
#[repr(C)]
struct IFileDialogV {
    base: IModalWindowV,
    set_file_types: unsafe extern "system" fn(
        this: *mut c_void,
        count_filetypes: u32,
        filter_spec: *const COMDLG_FILTERSPEC,
    ) -> HRESULT,
    __set_file_type_index: usize,
    __get_file_type_index: usize,
    __advise: usize,
    __unadvise: usize,
    set_options:
        unsafe extern "system" fn(this: *mut c_void, options: FILEOPENDIALOGOPTIONS) -> HRESULT,
    __get_options: usize,
    __set_default_folder: usize,
    set_folder: unsafe extern "system" fn(this: *mut c_void, shell_item: *mut c_void) -> HRESULT,
    __get_folder: usize,
    __get_current_selection: usize,
    set_file_name: unsafe extern "system" fn(this: *mut c_void, name: PCWSTR) -> HRESULT,
    __get_file_name: usize,
    set_title: unsafe extern "system" fn(this: *mut c_void, title: PCWSTR) -> HRESULT,
    __set_ok_button_label: usize,
    __set_file_name_label: usize,
    get_result:
        unsafe extern "system" fn(this: *mut c_void, shell_item: *mut IShellItem) -> HRESULT,
    __add_place: usize,
    set_default_extension:
        unsafe extern "system" fn(this: *mut c_void, default_ext: PCWSTR) -> HRESULT,
    __close: usize,
    __set_client_guid: usize,
    __clear_client_data: usize,
    __set_filter: usize,
}

#[repr(C)]
struct IFileDialog(*mut Interface<IFileDialogV>);

impl Drop for IFileDialog {
    fn drop(&mut self) {
        drop_impl(self.0.cast());
    }
}

/// <https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-ifileopendialog>
#[repr(C)]
struct IFileOpenDialogV {
    base: IFileDialogV,
    /// <https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ifileopendialog-getresults>
    get_results:
        unsafe extern "system" fn(this: *mut c_void, results: *mut IShellItemArray) -> HRESULT,
    __get_selected_items: usize,
}

struct IFileOpenDialog(*mut Interface<IFileOpenDialogV>);

impl Drop for IFileOpenDialog {
    fn drop(&mut self) {
        drop_impl(self.0.cast());
    }
}

enum DialogInner {
    Open(IFileOpenDialog),
    Save(IFileDialog),
}

impl DialogInner {
    unsafe fn new(open: bool) -> Result<Self> {
        const FILE_OPEN_DIALOG_IID: GUID = GUID::from_u128(0xd57c7288_d4ad_4768_be02_9d969532d960);
        const FILE_SAVE_DIALOG_IID: GUID = GUID::from_u128(0x84bccd23_5fde_4cdb_aea4_af64b83d78ab);

        unsafe {
            let (cls_id, iid) = if open {
                (&FileOpenDialog, &FILE_OPEN_DIALOG_IID)
            } else {
                (&FileSaveDialog, &FILE_SAVE_DIALOG_IID)
            };

            let mut iptr = std::mem::MaybeUninit::uninit();
            wrap_err(CoCreateInstance(
                cls_id,
                std::ptr::null_mut(),
                CLSCTX_INPROC_SERVER,
                iid,
                iptr.as_mut_ptr(),
            ))?;

            let iptr = iptr.assume_init();

            Ok(if open {
                Self::Open(IFileOpenDialog(iptr.cast()))
            } else {
                Self::Save(IFileDialog(iptr.cast()))
            })
        }
    }

    #[inline]
    unsafe fn open() -> Result<Self> {
        unsafe { Self::new(true) }
    }

    #[inline]
    unsafe fn save() -> Result<Self> {
        unsafe { Self::new(false) }
    }

    #[inline]
    unsafe fn fd(&self) -> (*mut std::ffi::c_void, &IFileDialogV) {
        match self {
            Self::Save(s) => unsafe { (s.0.cast(), (*s.0).vtbl()) },
            Self::Open(o) => unsafe { (o.0.cast(), &(*o.0).vtbl().base) },
        }
    }

    #[inline]
    unsafe fn set_options(&self, opts: FILEOPENDIALOGOPTIONS) -> Result<()> {
        let (d, v) = self.fd();
        wrap_err((v.set_options)(d, opts))
    }

    #[inline]
    unsafe fn set_title(&self, title: &[u16]) -> Result<()> {
        let (d, v) = self.fd();
        wrap_err((v.set_title)(d, title.as_ptr()))
    }

    #[inline]
    unsafe fn set_default_extension(&self, extension: &[u16]) -> Result<()> {
        let (d, v) = self.fd();
        wrap_err((v.set_default_extension)(d, extension.as_ptr()))
    }

    #[inline]
    unsafe fn set_file_types(&self, specs: &[COMDLG_FILTERSPEC]) -> Result<()> {
        let (d, v) = self.fd();
        wrap_err((v.set_file_types)(d, specs.len() as _, specs.as_ptr()))
    }

    #[inline]
    unsafe fn set_filename(&self, fname: &[u16]) -> Result<()> {
        let (d, v) = self.fd();
        wrap_err((v.set_file_name)(d, fname.as_ptr()))
    }

    #[inline]
    unsafe fn set_folder(&self, folder: &IShellItem) -> Result<()> {
        let (d, v) = self.fd();
        wrap_err((v.set_folder)(d, folder.0.cast()))
    }

    #[inline]
    unsafe fn show(&self, parent: Option<HWND>) -> Result<()> {
        let (d, v) = self.fd();
        wrap_err((v.base.show)(d, parent.unwrap_or_default()))
    }

    #[inline]
    unsafe fn get_result(&self) -> Result<PathBuf> {
        let (d, v) = self.fd();
        let mut res = std::mem::MaybeUninit::uninit();
        wrap_err((v.get_result)(d, res.as_mut_ptr()))?;
        let res = res.assume_init();
        res.get_path()
    }

    #[inline]
    unsafe fn get_results(&self) -> Result<Vec<PathBuf>> {
        let Self::Open(od) = self else { unreachable!() };

        let mut res = std::mem::MaybeUninit::uninit();
        wrap_err(((*(*od.0).vtable).get_results)(
            od.0.cast(),
            res.as_mut_ptr(),
        ))?;
        let items = res.assume_init();

        let sia = items.0.cast();
        let svt = &*(*items.0).vtable;

        let mut count = 0;
        wrap_err((svt.get_count)(sia, &mut count))?;

        let mut paths = Vec::with_capacity(count as usize);
        for index in 0..count {
            let mut item = std::mem::MaybeUninit::uninit();
            wrap_err((svt.get_item_at)(sia, index, item.as_mut_ptr()))?;
            let item = item.assume_init();

            let path = item.get_path()?;
            paths.push(path);
        }

        Ok(paths)
    }
}

pub struct IDialog(DialogInner, Option<HWND>);

impl IDialog {
    fn new_open_dialog(opt: &FileDialog) -> Result<Self> {
        let dialog = unsafe { DialogInner::open()? };

        let parent = match opt.parent {
            Some(RawWindowHandle::Win32(handle)) => Some(handle.hwnd as _),
            None => None,
            _ => unreachable!("unsupported window handle, expected: Windows"),
        };

        Ok(Self(dialog, parent))
    }

    fn new_save_dialog(opt: &FileDialog) -> Result<Self> {
        let dialog = unsafe { DialogInner::save()? };

        let parent = match opt.parent {
            Some(RawWindowHandle::Win32(handle)) => Some(handle.hwnd as _),
            None => None,
            _ => unreachable!("unsupported window handle, expected: Windows"),
        };

        Ok(Self(dialog, parent))
    }

    fn add_filters(&self, filters: &[crate::file_dialog::Filter]) -> Result<()> {
        {
            let Some(first_filter) = filters.first() else { return Ok(()) };
            if let Some(first_extension) = first_filter.extensions.first() {
                let extension = str_to_vec_u16(first_extension);
                unsafe { self.0.set_default_extension(&extension)? }
            }
        }

        let f_list = {
            let mut f_list = Vec::new();
            let mut ext_string = String::new();

            for f in filters.iter() {
                let name = str_to_vec_u16(&f.name);
                ext_string.clear();

                for ext in &f.extensions {
                    use std::fmt::Write;
                    // This is infallible for String (barring OOM)
                    let _ = write!(&mut ext_string, "*.{ext};");
                }

                // pop trailing ;
                ext_string.pop();

                f_list.push((name, str_to_vec_u16(&ext_string)));
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
            self.0.set_file_types(&spec)?;
        }
        Ok(())
    }

    fn set_path(&self, path: &Option<PathBuf>) -> Result<()> {
        const SHELL_ITEM_IID: GUID = GUID::from_u128(0x43826d1e_e718_42ee_bc55_a1e261c37bfe);

        let Some(path) = path.as_ref().and_then(|p| p.to_str()) else { return Ok(()) };

        // Strip Win32 namespace prefix from the path
        let path = path.strip_prefix(r"\\?\").unwrap_or(path);

        let wide_path = str_to_vec_u16(path);

        unsafe {
            let mut item = std::mem::MaybeUninit::uninit();
            if wrap_err(SHCreateItemFromParsingName(
                wide_path.as_ptr(),
                std::ptr::null_mut(),
                &SHELL_ITEM_IID,
                item.as_mut_ptr(),
            ))
            .is_ok()
            {
                let item = IShellItem(item.assume_init().cast());
                // For some reason SetDefaultFolder(), does not guarantees default path, so we use SetFolder
                self.0.set_folder(&item)?;
            }
        }

        Ok(())
    }

    fn set_file_name(&self, file_name: &Option<String>) -> Result<()> {
        if let Some(path) = file_name {
            let wide_path = str_to_vec_u16(path);

            unsafe {
                self.0.set_filename(&wide_path)?;
            }
        }
        Ok(())
    }

    fn set_title(&self, title: &Option<String>) -> Result<()> {
        if let Some(title) = title {
            let wide_title = str_to_vec_u16(title);

            unsafe {
                self.0.set_title(&wide_title)?;
            }
        }
        Ok(())
    }

    pub fn get_results(&self) -> Result<Vec<PathBuf>> {
        unsafe { self.0.get_results() }
    }

    pub fn get_result(&self) -> Result<PathBuf> {
        unsafe { self.0.get_result() }
    }

    pub fn show(&self) -> Result<()> {
        unsafe { self.0.show(self.1) }
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
            dialog.0.set_options(FOS_PICKFOLDERS)?;
        }

        Ok(dialog)
    }

    pub fn build_pick_folders(opt: &FileDialog) -> Result<Self> {
        let dialog = IDialog::new_open_dialog(opt)?;

        dialog.set_path(&opt.starting_directory)?;
        dialog.set_title(&opt.title)?;
        let opts = FOS_PICKFOLDERS | FOS_ALLOWMULTISELECT;

        unsafe {
            dialog.0.set_options(opts)?;
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
            dialog.0.set_options(FOS_ALLOWMULTISELECT)?;
        }

        Ok(dialog)
    }
}
