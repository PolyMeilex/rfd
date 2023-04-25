use std::ffi::c_void;
use windows_sys::core::{HRESULT, PCWSTR, PWSTR};
pub use windows_sys::Win32::{
    Foundation::HWND,
    UI::Shell::{Common::COMDLG_FILTERSPEC, FILEOPENDIALOGOPTIONS, SIGDN, SIGDN_FILESYSPATH},
};

pub(crate) type Result<T> = std::result::Result<T, HRESULT>;

#[inline]
pub(super) fn wrap_err(hresult: HRESULT) -> Result<()> {
    if hresult >= 0 {
        Ok(())
    } else {
        Err(hresult)
    }
}

#[inline]
pub(super) unsafe fn read_to_string(ptr: *const u16) -> String {
    let mut cursor = ptr;

    while *cursor != 0 {
        cursor = cursor.add(1);
    }

    let slice = std::slice::from_raw_parts(ptr, cursor.offset_from(ptr) as usize);
    String::from_utf16(slice).unwrap()
}

#[repr(C)]
pub(super) struct Interface<T> {
    vtable: *mut T,
}

impl<T> Interface<T> {
    #[inline]
    pub(super) fn vtbl(&self) -> &T {
        unsafe { &*self.vtable }
    }
}

#[repr(C)]
pub(super) struct IUnknownV {
    __query_interface: usize,
    __add_ref: usize,
    pub(super) release: unsafe extern "system" fn(this: *mut c_void) -> u32,
}

pub(super) type IUnknown = Interface<IUnknownV>;

#[inline]
fn drop_impl(ptr: *mut c_void) {
    unsafe {
        ((*ptr.cast::<IUnknown>()).vtbl().release)(ptr);
    }
}

#[repr(C)]
pub(super) struct IShellItemV {
    base: IUnknownV,
    __bind_to_handler: usize,
    __get_parent: usize,
    pub(super) get_display_name:
        unsafe extern "system" fn(this: *mut c_void, name_look: SIGDN, name: *mut PWSTR) -> HRESULT,
    __get_attributes: usize,
    __compare: usize,
}

#[repr(transparent)]
pub(super) struct IShellItem(pub(super) *mut Interface<IShellItemV>);

impl IShellItem {
    pub(super) fn get_path(&self) -> Result<std::path::PathBuf> {
        let filename = unsafe {
            let mut dname = std::mem::MaybeUninit::uninit();
            wrap_err(((*self.0).vtbl().get_display_name)(
                self.0.cast(),
                SIGDN_FILESYSPATH,
                dname.as_mut_ptr(),
            ))?;

            let dname = dname.assume_init();
            let fname = read_to_string(dname);
            windows_sys::Win32::System::Com::CoTaskMemFree(dname.cast());
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
    pub(super) get_count:
        unsafe extern "system" fn(this: *mut c_void, num_items: *mut u32) -> HRESULT,
    pub(super) get_item_at: unsafe extern "system" fn(
        this: *mut c_void,
        dwindex: u32,
        ppsi: *mut IShellItem,
    ) -> HRESULT,
    __enum_items: usize,
}

#[repr(transparent)]
pub(super) struct IShellItemArray(*mut Interface<IShellItemArrayV>);

impl IShellItemArray {
    #[inline]
    pub(super) fn get_count(&self) -> Result<u32> {
        let mut count = 0;
        unsafe {
            wrap_err(((*self.0).vtbl().get_count)(self.0.cast(), &mut count))?;
        }
        Ok(count)
    }

    #[inline]
    pub(super) fn get_item_at(&self, index: u32) -> Result<IShellItem> {
        let mut item = std::mem::MaybeUninit::uninit();
        unsafe {
            wrap_err(((*self.0).vtbl().get_item_at)(
                self.0.cast(),
                index,
                item.as_mut_ptr(),
            ))?;
            Ok(item.assume_init())
        }
    }
}

impl Drop for IShellItemArray {
    fn drop(&mut self) {
        drop_impl(self.0.cast());
    }
}

#[repr(C)]
pub(super) struct IModalWindowV {
    base: IUnknownV,
    pub(super) show: unsafe extern "system" fn(this: *mut c_void, owner: HWND) -> HRESULT,
}

/// <https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-ifiledialog>
#[repr(C)]
pub(super) struct IFileDialogV {
    pub(super) base: IModalWindowV,
    pub(super) set_file_types: unsafe extern "system" fn(
        this: *mut c_void,
        count_filetypes: u32,
        filter_spec: *const COMDLG_FILTERSPEC,
    ) -> HRESULT,
    __set_file_type_index: usize,
    __get_file_type_index: usize,
    __advise: usize,
    __unadvise: usize,
    pub(super) set_options:
        unsafe extern "system" fn(this: *mut c_void, options: FILEOPENDIALOGOPTIONS) -> HRESULT,
    __get_options: usize,
    __set_default_folder: usize,
    pub(super) set_folder:
        unsafe extern "system" fn(this: *mut c_void, shell_item: *mut c_void) -> HRESULT,
    __get_folder: usize,
    __get_current_selection: usize,
    pub(super) set_file_name: unsafe extern "system" fn(this: *mut c_void, name: PCWSTR) -> HRESULT,
    __get_file_name: usize,
    pub(super) set_title: unsafe extern "system" fn(this: *mut c_void, title: PCWSTR) -> HRESULT,
    __set_ok_button_label: usize,
    __set_file_name_label: usize,
    pub(super) get_result:
        unsafe extern "system" fn(this: *mut c_void, shell_item: *mut IShellItem) -> HRESULT,
    __add_place: usize,
    pub(super) set_default_extension:
        unsafe extern "system" fn(this: *mut c_void, default_ext: PCWSTR) -> HRESULT,
    __close: usize,
    __set_client_guid: usize,
    __clear_client_data: usize,
    __set_filter: usize,
}

#[repr(transparent)]
pub(super) struct IFileDialog(pub(super) *mut Interface<IFileDialogV>);

impl Drop for IFileDialog {
    fn drop(&mut self) {
        drop_impl(self.0.cast());
    }
}

/// <https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-ifileopendialog>
#[repr(C)]
pub(super) struct IFileOpenDialogV {
    pub(super) base: IFileDialogV,
    /// <https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ifileopendialog-getresults>
    pub(super) get_results:
        unsafe extern "system" fn(this: *mut c_void, results: *mut IShellItemArray) -> HRESULT,
    __get_selected_items: usize,
}

#[repr(transparent)]
pub(super) struct IFileOpenDialog(pub(super) *mut Interface<IFileOpenDialogV>);

impl IFileOpenDialog {
    #[inline]
    pub(super) fn get_results(&self) -> Result<IShellItemArray> {
        let mut res = std::mem::MaybeUninit::uninit();
        unsafe {
            wrap_err((((*self.0).vtbl()).get_results)(
                self.0.cast(),
                res.as_mut_ptr(),
            ))?;
            Ok(res.assume_init())
        }
    }
}

impl Drop for IFileOpenDialog {
    fn drop(&mut self) {
        drop_impl(self.0.cast());
    }
}
