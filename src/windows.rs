use crate::DialogParams;

use std::{ffi::OsStr, iter::once, mem, os::windows::ffi::OsStrExt, path::PathBuf};

use winapi::um::commdlg::{
    GetOpenFileNameW, OFN_FILEMUSTEXIST, OFN_NOCHANGEDIR, OFN_OVERWRITEPROMPT, OFN_PATHMUSTEXIST,
    OPENFILENAMEW,
};

extern "C" {
    fn wcslen(buf: *const u16) -> usize;
}

pub fn open_file_with_params(params: DialogParams) -> Option<PathBuf> {
    let mut filter = String::new();

    for f in params.filters.iter() {
        filter += &format!("{}\0{}\0", f.0, f.1);
    }

    unsafe {
        // This vec needs to be initialized with zeros, so we do not use `Vec::with_capacity` here
        let mut path: Vec<u16> = vec![0; 260];

        let filter: Vec<u16> = OsStr::new(&filter).encode_wide().chain(once(0)).collect();

        let mut ofn: OPENFILENAMEW = std::mem::zeroed();
        ofn.lStructSize = mem::size_of::<OPENFILENAMEW>() as u32;
        ofn.hwndOwner = std::mem::zeroed();

        ofn.lpstrFile = path.as_mut_ptr();
        ofn.nMaxFile = path.len() as _;

        if !params.filters.is_empty() {
            ofn.lpstrFilter = filter.as_ptr();
            ofn.nFilterIndex = 1;
        }

        ofn.Flags = OFN_OVERWRITEPROMPT | OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST | OFN_NOCHANGEDIR;

        let out = GetOpenFileNameW(&mut ofn);

        if out == 1 {
            let l = wcslen(ofn.lpstrFile);
            // Trim string
            path.set_len(l);

            String::from_utf16(&path).ok().map(PathBuf::from)
        } else {
            None
        }
    }
}

pub fn save_file_with_params(params: DialogParams) -> Option<PathBuf> {
    unimplemented!("save_file_with_params");
}

pub fn pick_folder() -> Option<PathBuf> {
    unimplemented!("pick_folder");
}

pub fn open_multiple_files_with_params(params: DialogParams) -> Option<Vec<PathBuf>> {
    unimplemented!("open_multiple_with_params");
}
