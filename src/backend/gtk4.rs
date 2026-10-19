use crate::{
    backend::DialogFutureType, file_dialog::Filter, message_dialog::MessageDialog, FileDialog,
    FileHandle, MessageButtons, MessageDialogResult,
};

use gtk4::{gio, glib, prelude::*, Window};

use std::{
    future::Future,
    path::{Path, PathBuf},
    sync::{mpsc, Once},
};

/// Makes sure something is running the GTK event loop
fn ensure_gtk_thread() {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        if unsafe { gtk4::ffi::gtk_is_initialized() } == glib::ffi::GFALSE {
            let (tx, rx) = mpsc::channel();

            std::thread::spawn(move || {
                // Acquires the default main context for this thread
                let res = gtk4::init();
                let ok = res.is_ok();
                tx.send(res).ok();
                if ok {
                    glib::MainLoop::new(None, false).run();
                }
            });

            rx.recv()
                .expect("GTK4 thread exited before initializing")
                .expect("failed to initialize GTK4");
        } else {
            // GTK was initialized by the application, possibly through the C API, so let gtk4-rs know about it on the GTK thread
            glib::idle_add_once(|| {
                // SAFETY: GTK is initialized, and we are on the gtk thread
                unsafe { gtk4::set_initialized() };
            });
        }
    });
}

/// Runs the future on GTK thread
fn run_on_main<R, Fut, F, D>(f: F, done: D)
where
    Fut: Future<Output = R> + 'static,
    F: FnOnce() -> Fut + Send + 'static,
    D: FnOnce(R) + Send + 'static,
{
    ensure_gtk_thread();

    glib::idle_add_once(move || {
        let fut = f();
        glib::MainContext::default().spawn_local(async move { done(fut.await) });
    });
}

/// Runs the future on GTK thread.
fn spawn<R, Fut, F>(f: F) -> DialogFutureType<R>
where
    R: Send + 'static,
    Fut: Future<Output = R> + 'static,
    F: FnOnce() -> Fut + Send + 'static,
{
    let (tx, rx) = crate::oneshot::channel();
    run_on_main(f, move |res| {
        tx.send(res).ok();
    });
    Box::pin(async move { rx.await.expect("GTK4 dialog task was dropped") })
}

fn block_on<R, Fut, F>(f: F) -> R
where
    R: Send + 'static,
    Fut: Future<Output = R> + 'static,
    F: FnOnce() -> Fut + Send + 'static,
{
    let context = glib::MainContext::default();
    if context.is_owner() {
        // We are already on GTK thread, so instead we run a nested main loop until the dialog responds
        ensure_gtk_thread();
        // SAFETY: GTK is initialized, and we are on the gtk thread
        unsafe { gtk4::set_initialized() };
        return context.block_on(f());
    }

    let (tx, rx) = mpsc::channel();
    run_on_main(f, move |res| {
        tx.send(res).ok();
    });
    rx.recv().expect("GTK4 dialog task was dropped")
}

fn path_to_file(path: &Path) -> gio::File {
    gio::File::for_path(path)
}

fn first_path(file: gio::File) -> Option<PathBuf> {
    file.path()
}

fn all_paths(files: gio::ListModel) -> Vec<PathBuf> {
    let mut paths = Vec::with_capacity(files.n_items() as usize);

    for index in 0..files.n_items() {
        let file = files
            .item(index)
            .and_downcast::<gio::File>()
            .and_then(|file| file.path());

        if let Some(path) = file {
            paths.push(path);
        }
    }

    paths
}

fn add_filters(dialog: &gtk4::FileDialog, filters: &[Filter]) {
    if filters.is_empty() {
        return;
    }

    let list = gio::ListStore::new::<gtk4::FileFilter>();
    let mut first_filter = None;

    for filter in filters {
        let gtk_filter = gtk4::FileFilter::new();
        gtk_filter.set_name(Some(&filter.name));

        for extension in &filter.extensions {
            if extension == "*" || extension.is_empty() {
                gtk_filter.add_pattern("*");
            } else {
                gtk_filter.add_pattern(&format!("*.{extension}"));
            }
        }

        if first_filter.is_none() {
            first_filter = Some(gtk_filter.clone());
        }

        list.append(&gtk_filter);
    }

    dialog.set_filters(Some(&list));
    if let Some(first_filter) = first_filter.as_ref() {
        dialog.set_default_filter(Some(first_filter));
    }
}

fn set_directory(dialog: &gtk4::FileDialog, directory: Option<&Path>) {
    if let Some(directory) = directory {
        dialog.set_initial_folder(Some(&path_to_file(directory)));
    }
}

fn set_file_name(dialog: &gtk4::FileDialog, directory: Option<&Path>, file_name: Option<&str>) {
    match (directory, file_name) {
        (Some(directory), Some(file_name)) => {
            let path = directory.join(file_name);
            dialog.set_initial_file(Some(&path_to_file(&path)));
        }
        (None, Some(file_name)) => dialog.set_initial_name(Some(file_name)),
        _ => {}
    }
}

fn set_save_name(dialog: &gtk4::FileDialog, directory: Option<&Path>, file_name: Option<&str>) {
    match (directory, file_name) {
        (Some(directory), Some(file_name)) => {
            let path = directory.join(file_name);
            if path.exists() {
                dialog.set_initial_file(Some(&path_to_file(&path)));
            } else {
                dialog.set_initial_name(Some(file_name));
            }
        }
        (_, Some(file_name)) => dialog.set_initial_name(Some(file_name)),
        _ => {}
    }
}

fn configure_dialog(dialog: &gtk4::FileDialog, options: &FileDialog) {
    add_filters(dialog, &options.filters);
    set_directory(dialog, options.starting_directory.as_deref());
    dialog.set_modal(true);
}

fn build_file_dialog(options: &FileDialog, title: &str) -> gtk4::FileDialog {
    let dialog = gtk4::FileDialog::new();
    dialog.set_title(options.title.as_deref().unwrap_or(title));
    configure_dialog(&dialog, options);
    dialog
}

fn build_pick_file(options: &FileDialog) -> gtk4::FileDialog {
    let dialog = build_file_dialog(options, "Open File");
    set_file_name(
        &dialog,
        options.starting_directory.as_deref(),
        options.file_name.as_deref(),
    );
    dialog
}

fn build_pick_files(options: &FileDialog) -> gtk4::FileDialog {
    let dialog = build_file_dialog(options, "Open File");
    set_file_name(
        &dialog,
        options.starting_directory.as_deref(),
        options.file_name.as_deref(),
    );
    dialog
}

fn build_pick_folder(options: &FileDialog) -> gtk4::FileDialog {
    let dialog = build_file_dialog(options, "Select Folder");
    set_file_name(
        &dialog,
        options.starting_directory.as_deref(),
        options.file_name.as_deref(),
    );
    dialog
}

fn build_pick_folders(options: &FileDialog) -> gtk4::FileDialog {
    let dialog = build_file_dialog(options, "Select Folder");
    set_file_name(
        &dialog,
        options.starting_directory.as_deref(),
        options.file_name.as_deref(),
    );
    dialog
}

fn build_save_file(options: &FileDialog) -> gtk4::FileDialog {
    let dialog = build_file_dialog(options, "Save File");
    set_save_name(
        &dialog,
        options.starting_directory.as_deref(),
        options.file_name.as_deref(),
    );
    dialog
}

async fn pick_file(options: FileDialog) -> Option<PathBuf> {
    build_pick_file(&options)
        .open_future(None::<&Window>)
        .await
        .ok()
        .and_then(first_path)
}

async fn pick_files(options: FileDialog) -> Option<Vec<PathBuf>> {
    build_pick_files(&options)
        .open_multiple_future(None::<&Window>)
        .await
        .ok()
        .map(all_paths)
}

async fn pick_folder(options: FileDialog) -> Option<PathBuf> {
    build_pick_folder(&options)
        .select_folder_future(None::<&Window>)
        .await
        .ok()
        .and_then(first_path)
}

async fn pick_folders(options: FileDialog) -> Option<Vec<PathBuf>> {
    build_pick_folders(&options)
        .select_multiple_folders_future(None::<&Window>)
        .await
        .ok()
        .map(all_paths)
}

async fn save_file(options: FileDialog) -> Option<PathBuf> {
    build_save_file(&options)
        .save_future(None::<&Window>)
        .await
        .ok()
        .and_then(first_path)
}

fn message_result(buttons: &MessageButtons, response: i32) -> MessageDialogResult {
    use MessageButtons::*;

    match (buttons, response) {
        (Ok, 1) => MessageDialogResult::Cancel,
        (Ok, 0) => MessageDialogResult::Ok,
        (OkCancel, 0) => MessageDialogResult::Cancel,
        (OkCancel, 1) => MessageDialogResult::Ok,
        (YesNo, 0) => MessageDialogResult::No,
        (YesNo, 1) => MessageDialogResult::Yes,
        (YesNoCancel, 0) => MessageDialogResult::Cancel,
        (YesNoCancel, 1) => MessageDialogResult::Yes,
        (YesNoCancel, 2) => MessageDialogResult::No,
        (OkCustom(_), 1) => MessageDialogResult::Cancel,
        (OkCustom(custom), 0) => MessageDialogResult::Custom(custom.to_owned()),
        (OkCancelCustom(_, custom), 0) => MessageDialogResult::Custom(custom.to_owned()),
        (OkCancelCustom(custom, _), 1) => MessageDialogResult::Custom(custom.to_owned()),
        (YesNoCancelCustom(_, _, custom), 0) => MessageDialogResult::Custom(custom.to_owned()),
        (YesNoCancelCustom(custom, _, _), 1) => MessageDialogResult::Custom(custom.to_owned()),
        (YesNoCancelCustom(_, custom, _), 2) => MessageDialogResult::Custom(custom.to_owned()),
        _ => MessageDialogResult::Cancel,
    }
}

async fn run_message_dialog(options: MessageDialog) -> MessageDialogResult {
    let dialog = gtk4::AlertDialog::builder()
        .modal(true)
        .message(&options.title)
        .detail(&options.description)
        .build();

    dialog.set_default_button(1);
    dialog.set_cancel_button(0);

    match &options.buttons {
        MessageButtons::Ok => {
            dialog.set_buttons(&["OK"]);
            dialog.set_default_button(0);
            dialog.set_cancel_button(1);
        }
        MessageButtons::OkCancel => {
            dialog.set_buttons(&["Cancel", "OK"]);
        }
        MessageButtons::YesNo => {
            dialog.set_buttons(&["No", "Yes"]);
        }
        MessageButtons::YesNoCancel => {
            dialog.set_buttons(&["Cancel", "Yes", "No"]);
        }
        MessageButtons::OkCustom(ok) => {
            dialog.set_buttons(&[ok.as_str()]);
            dialog.set_default_button(0);
            dialog.set_cancel_button(1);
        }
        MessageButtons::OkCancelCustom(ok, cancel) => {
            dialog.set_buttons(&[cancel.as_str(), ok.as_str()]);
        }
        MessageButtons::YesNoCancelCustom(yes, no, cancel) => {
            dialog.set_buttons(&[cancel.as_str(), yes.as_str(), no.as_str()]);
        }
    }

    let single_button = matches!(
        options.buttons,
        MessageButtons::Ok | MessageButtons::OkCustom(_)
    );

    let response = match dialog.choose_future(None::<&Window>).await {
        // for one button dialog, 1 is cancel and 0 is ok
        Ok(_) if single_button => 0,
        Ok(response) => response,
        Err(_) if single_button => 1,
        Err(_) => 0,
    };

    message_result(&options.buttons, response)
}

use crate::backend::FilePickerDialogImpl;
impl FilePickerDialogImpl for FileDialog {
    fn pick_file(self) -> Option<PathBuf> {
        block_on(move || pick_file(self))
    }

    fn pick_files(self) -> Option<Vec<PathBuf>> {
        block_on(move || pick_files(self))
    }
}

use crate::backend::AsyncFilePickerDialogImpl;
impl AsyncFilePickerDialogImpl for FileDialog {
    fn pick_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        spawn(move || async move { pick_file(self).await.map(FileHandle::wrap) })
    }

    fn pick_files_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        spawn(move || async move {
            pick_files(self)
                .await
                .map(|files| files.into_iter().map(FileHandle::wrap).collect())
        })
    }
}

use crate::backend::FolderPickerDialogImpl;
impl FolderPickerDialogImpl for FileDialog {
    fn pick_folder(self) -> Option<PathBuf> {
        block_on(move || pick_folder(self))
    }

    fn pick_folders(self) -> Option<Vec<PathBuf>> {
        block_on(move || pick_folders(self))
    }
}

use crate::backend::AsyncFolderPickerDialogImpl;
impl AsyncFolderPickerDialogImpl for FileDialog {
    fn pick_folder_async(self) -> DialogFutureType<Option<FileHandle>> {
        spawn(move || async move { pick_folder(self).await.map(FileHandle::wrap) })
    }

    fn pick_folders_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        spawn(move || async move {
            pick_folders(self)
                .await
                .map(|folders| folders.into_iter().map(FileHandle::wrap).collect())
        })
    }
}

use crate::backend::FileSaveDialogImpl;
impl FileSaveDialogImpl for FileDialog {
    fn save_file(self) -> Option<PathBuf> {
        block_on(move || save_file(self))
    }
}

use crate::backend::AsyncFileSaveDialogImpl;
impl AsyncFileSaveDialogImpl for FileDialog {
    fn save_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        spawn(move || async move { save_file(self).await.map(FileHandle::wrap) })
    }
}

use crate::backend::MessageDialogImpl;
impl MessageDialogImpl for MessageDialog {
    fn show(self) -> MessageDialogResult {
        block_on(move || run_message_dialog(self))
    }
}

use crate::backend::AsyncMessageDialogImpl;
impl AsyncMessageDialogImpl for MessageDialog {
    fn show_async(self) -> DialogFutureType<MessageDialogResult> {
        spawn(move || run_message_dialog(self))
    }
}
