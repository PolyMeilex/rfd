use std::path::PathBuf;
use std::process::Command;

use super::*;

mod paths;
use crate::{FileDialog, FileHandle, MessageButtons, MessageDialog, MessageLevel};
use paths::parse_paths;

fn choose(dialog: FileDialog, save: bool, folders: bool, multiple: bool) -> Option<Vec<PathBuf>> {
    let mut command = Command::new("/boot/system/bin/filepanel");
    command.arg(if save { "--save" } else { "--load" });
    if !multiple {
        command.arg("--single");
    }
    command.args(["--kind", if folders { "d" } else { "f" }]);
    if let Some(directory) = dialog.starting_directory {
        command.arg("--directory").arg(directory);
    }
    if let Some(title) = dialog.title {
        command.arg("--title").arg(title);
    }
    if save {
        if let Some(name) = dialog.file_name {
            command.arg("--name").arg(name);
        }
    }
    let output = command.output().ok()?;
    parse_paths(output, save)
}

fn first(paths: Option<Vec<PathBuf>>) -> Option<PathBuf> {
    paths?.into_iter().next()
}

fn background<T: Send + Default + 'static>(
    run: impl FnOnce() -> T + Send + 'static,
) -> DialogFutureType<T> {
    let (sender, receiver) = crate::oneshot::channel();
    let _ = std::thread::Builder::new()
        .name("rfd dialog".into())
        .spawn(move || {
            let _ = sender.send(run());
        });
    Box::pin(async move { receiver.await.unwrap_or_default() })
}

impl FilePickerDialogImpl for FileDialog {
    fn pick_file(self) -> Option<PathBuf> {
        first(choose(self, false, false, false))
    }
    fn pick_files(self) -> Option<Vec<PathBuf>> {
        choose(self, false, false, true)
    }
}
impl FolderPickerDialogImpl for FileDialog {
    fn pick_folder(self) -> Option<PathBuf> {
        first(choose(self, false, true, false))
    }
    fn pick_folders(self) -> Option<Vec<PathBuf>> {
        choose(self, false, true, true)
    }
}
impl FileSaveDialogImpl for FileDialog {
    fn save_file(self) -> Option<PathBuf> {
        first(choose(self, true, false, false))
    }
}
impl AsyncFilePickerDialogImpl for FileDialog {
    fn pick_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        background(move || self.pick_file().map(FileHandle::wrap))
    }
    fn pick_files_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        background(move || {
            self.pick_files()
                .map(|paths| paths.into_iter().map(FileHandle::wrap).collect())
        })
    }
}
impl AsyncFolderPickerDialogImpl for FileDialog {
    fn pick_folder_async(self) -> DialogFutureType<Option<FileHandle>> {
        background(move || self.pick_folder().map(FileHandle::wrap))
    }
    fn pick_folders_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        background(move || {
            self.pick_folders()
                .map(|paths| paths.into_iter().map(FileHandle::wrap).collect())
        })
    }
}
impl AsyncFileSaveDialogImpl for FileDialog {
    fn save_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        background(move || self.save_file().map(FileHandle::wrap))
    }
}
impl MessageDialogImpl for MessageDialog {
    fn show(self) -> MessageDialogResult {
        use MessageDialogResult as Result;
        let buttons: Vec<(String, Result)> = match self.buttons {
            MessageButtons::Ok => vec![("OK".into(), Result::Ok)],
            MessageButtons::OkCancel => {
                vec![("OK".into(), Result::Ok), ("Cancel".into(), Result::Cancel)]
            }
            MessageButtons::YesNo => vec![("Yes".into(), Result::Yes), ("No".into(), Result::No)],
            MessageButtons::YesNoCancel => vec![
                ("Yes".into(), Result::Yes),
                ("No".into(), Result::No),
                ("Cancel".into(), Result::Cancel),
            ],
            MessageButtons::OkCustom(a) => vec![(a.clone(), Result::Custom(a))],
            MessageButtons::OkCancelCustom(a, b) => vec![
                (a.clone(), Result::Custom(a)),
                (b.clone(), Result::Custom(b)),
            ],
            MessageButtons::YesNoCancelCustom(a, b, c) => vec![
                (a.clone(), Result::Custom(a)),
                (b.clone(), Result::Custom(b)),
                (c.clone(), Result::Custom(c)),
            ],
        };
        let level = match self.level {
            MessageLevel::Info => "--info",
            MessageLevel::Warning => "--warning",
            MessageLevel::Error => "--stop",
        };
        let output = Command::new("/boot/system/bin/alert")
            .arg(level)
            .arg(format!("{}\n\n{}", self.title, self.description))
            .args(buttons.iter().map(|(label, _)| label))
            .output();
        output
            .ok()
            .and_then(|out| out.status.code())
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| buttons.get(index))
            .map(|(_, result)| result.clone())
            .unwrap_or_default()
    }
}
impl AsyncMessageDialogImpl for MessageDialog {
    fn show_async(self) -> DialogFutureType<MessageDialogResult> {
        background(move || self.show())
    }
}
