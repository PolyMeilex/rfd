use crate::DialogParams;
use std::path::PathBuf;

extern "C" {
    fn wcslen(buf: *const u16) -> usize;
}

pub fn open_with_params(params: DialogParams) -> Option<PathBuf> {
    use winapi::shared::windef::HWND;
    use winapi::um::commdlg::GetOpenFileNameW;
    use winapi::um::commdlg::OPENFILENAMEW;

    let size = std::mem::size_of::<OPENFILENAMEW>() as u32;

    let mut path = String::new();

    for f in params.filters.iter() {
        path += &format!("{}\0{}\0", f.0, f.1);
    }

    let out = unsafe {
        // This vec needs to be initialized with zeros, so we do not use `Vec::with_capacity` here
        let mut name: Vec<u16> = vec![0; 260];
        use std::ffi::OsStr;
        use std::iter::once;
        use std::os::windows::ffi::OsStrExt;

        let lpstrFilter: Vec<u16> = OsStr::new(&path).encode_wide().chain(once(0)).collect();

        let mut ofn: OPENFILENAMEW = std::mem::zeroed();
        ofn.lStructSize = size;
        ofn.lpstrFile = name.as_mut_ptr();
        ofn.lpstrFilter = lpstrFilter.as_ptr();
        ofn.nMaxFile = 260;
        ofn.hwndOwner = std::mem::zeroed();

        let out = GetOpenFileNameW(&mut ofn);

        if out == 1 {
            name.set_len(wcslen(ofn.lpstrFile));

            String::from_utf16(&name).ok()
        } else {
            None
        }
    };

    if let Some(out) = out {
        Some(PathBuf::from(out))
    } else {
        None
    }
}

pub fn open_multiple_files_with_params(params: DialogParams) -> Option<Vec<PathBuf>> {
    unimplemented!("open_multiple_with_params");
}
