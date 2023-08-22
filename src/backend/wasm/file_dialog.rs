//
// File Save
//

use crate::{
    backend::{AsyncFileSaveDialogImpl, DialogFutureType},
    file_dialog::FileDialog,
    FileHandle,
};
use std::future::ready;
impl AsyncFileSaveDialogImpl for FileDialog {
    fn save_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        let file = FileHandle::default_with_name(&self.file_name.unwrap_or_default());

        Box::pin(ready(Some(file)))
    }
}
