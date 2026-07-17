//
// File Save
//

use crate::{
    backend::{
        wasm::{check_exists, FileKind, WasmDialog},
        AsyncFileSaveDialogImpl, DialogFutureType,
    },
    file_dialog::FileDialog,
    FileHandle,
};
use std::future::ready;

impl AsyncFileSaveDialogImpl for FileDialog {
    fn save_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        let window = web_sys::window().expect("Window not found");
        if check_exists(&window, "showSaveFilePicker") {
            let dialog = WasmDialog::new(&FileKind::OutEarly(self));
            Box::pin(dialog.try_open_writable_file())
        } else {
            let file = FileHandle::writable(self);
            Box::pin(ready(Some(file)))
        }
    }
}
