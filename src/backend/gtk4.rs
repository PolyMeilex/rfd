use crate::MessageButtons;
use crate::backend::DialogFutureType;
use crate::file_dialog::Filter;
use crate::message_dialog::MessageDialog;
use crate::{FileDialog, FileHandle, MessageDialogResult};
use gtk4::{Window, gio, glib, prelude::*};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex, OnceLock};

static GTK_THREAD: OnceLock<GtkGlobalThread> = OnceLock::new();

type GtkJob = Box<dyn FnOnce() + Send + 'static>;

struct GtkGlobalThread {
    context: glib::MainContext,
    running: Arc<AtomicBool>,
    sender: mpsc::Sender<GtkJob>,
}

impl GtkGlobalThread {
    fn instance() -> &'static Self {
        GTK_THREAD.get_or_init(Self::new)
    }

    fn new() -> Self {
        let context = glib::MainContext::default();
        let running = Arc::new(AtomicBool::new(true));
        let thread_context = context.clone();
        let thread_running = Arc::clone(&running);
        let (sender, receiver) = mpsc::channel::<GtkJob>();

        std::thread::spawn(move || {
            let _guard = thread_context
                .acquire()
                .expect("failed to acquire GTK main context");
            thread_context
                .with_thread_default(|| {
                    let _ = gtk4::init();

                    while thread_running.load(Ordering::Acquire) {
                        let job = match receiver.recv() {
                            Ok(job) => job,
                            Err(_) => break,
                        };
                        job();
                    }
                })
                .expect("failed to set GTK main context as thread default");
        });

        Self {
            context,
            running,
            sender,
        }
    }

    fn run_blocking<T, F>(&self, cb: F) -> T
    where
        T: Send + Clone + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        let result = Arc::new((Mutex::new(None), Condvar::new()));
        let result_clone = Arc::clone(&result);

        self.sender
            .send(Box::new(move || {
                let value = cb();
                let (lock, cvar) = &*result_clone;
                *lock.lock().unwrap() = Some(value);
                cvar.notify_all();
            }))
            .expect("failed to send GTK job to GTK thread");

        let (lock, cvar) = &*result;
        let result = cvar
            .wait_while(lock.lock().unwrap(), |value| value.is_none())
            .unwrap();
        result.clone().unwrap()
    }
}

impl Drop for GtkGlobalThread {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Release);
        let _ = self.sender.send(Box::new(|| {}));
        self.context.wakeup();
    }
}

fn async_thread<T, F>(f: F) -> DialogFutureType<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    Box::pin(async move {
        let (tx, rx) = crate::oneshot::channel();

        std::thread::spawn(move || {
            tx.send(f()).ok();
        });

        rx.await
            .expect("gtk4 worker thread dropped before returning a result")
    })
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

fn run_pick_file(dialog: gtk4::FileDialog) -> Option<PathBuf> {
    let context = glib::MainContext::ref_thread_default();
    context
        .block_on(dialog.open_future(Some(&Window::builder().title("dummy").build())))
        .ok()
        .and_then(first_path)
}

fn run_pick_files(dialog: gtk4::FileDialog) -> Option<Vec<PathBuf>> {
    let context = glib::MainContext::ref_thread_default();
    context
        .block_on(dialog.open_multiple_future(Some(&Window::builder().title("dummy").build())))
        .ok()
        .map(all_paths)
}

fn run_pick_folder(dialog: gtk4::FileDialog) -> Option<PathBuf> {
    let context = glib::MainContext::ref_thread_default();
    context
        .block_on(dialog.select_folder_future(Some(&Window::builder().title("dummy").build())))
        .ok()
        .and_then(first_path)
}

fn run_pick_folders(dialog: gtk4::FileDialog) -> Option<Vec<PathBuf>> {
    let context = glib::MainContext::ref_thread_default();
    context
        .block_on(
            dialog.select_multiple_folders_future(Some(&Window::builder().title("dummy").build())),
        )
        .ok()
        .map(all_paths)
}

fn run_save_file(dialog: gtk4::FileDialog) -> Option<PathBuf> {
    let context = glib::MainContext::ref_thread_default();
    context
        .block_on(dialog.save_future(Some(&Window::builder().title("dummy").build())))
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

fn run_message_dialog(options: MessageDialog) -> MessageDialogResult {
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

    let context = glib::MainContext::ref_thread_default();
    let response = context
        .block_on(dialog.choose_future(Some(&Window::builder().title("dummy").build())))
        .map(|result| match options.buttons {
            MessageButtons::Ok | MessageButtons::OkCustom(_) => match result {
                _ => 0,
            },
            _ => result,
        })
        .unwrap_or(match options.buttons {
            // for one button dialog, 1 is cancel and 0 is ok
            MessageButtons::Ok | MessageButtons::OkCustom(_) => 1,
            _ => 0,
        });

    message_result(&options.buttons, response)
}

use crate::backend::FilePickerDialogImpl;
impl FilePickerDialogImpl for FileDialog {
    fn pick_file(self) -> Option<PathBuf> {
        GtkGlobalThread::instance().run_blocking(move || run_pick_file(build_pick_file(&self)))
    }

    fn pick_files(self) -> Option<Vec<PathBuf>> {
        GtkGlobalThread::instance().run_blocking(move || run_pick_files(build_pick_files(&self)))
    }
}

use crate::backend::AsyncFilePickerDialogImpl;
impl AsyncFilePickerDialogImpl for FileDialog {
    fn pick_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        async_thread(move || Self::pick_file(self).map(FileHandle::wrap))
    }

    fn pick_files_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        async_thread(move || {
            Self::pick_files(self).map(|files| files.into_iter().map(FileHandle::wrap).collect())
        })
    }
}

use crate::backend::FolderPickerDialogImpl;
impl FolderPickerDialogImpl for FileDialog {
    fn pick_folder(self) -> Option<PathBuf> {
        GtkGlobalThread::instance().run_blocking(move || run_pick_folder(build_pick_folder(&self)))
    }

    fn pick_folders(self) -> Option<Vec<PathBuf>> {
        GtkGlobalThread::instance()
            .run_blocking(move || run_pick_folders(build_pick_folders(&self)))
    }
}

use crate::backend::AsyncFolderPickerDialogImpl;
impl AsyncFolderPickerDialogImpl for FileDialog {
    fn pick_folder_async(self) -> DialogFutureType<Option<FileHandle>> {
        async_thread(move || Self::pick_folder(self).map(FileHandle::wrap))
    }

    fn pick_folders_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        async_thread(move || {
            Self::pick_folders(self)
                .map(|folders| folders.into_iter().map(FileHandle::wrap).collect())
        })
    }
}

use crate::backend::FileSaveDialogImpl;
impl FileSaveDialogImpl for FileDialog {
    fn save_file(self) -> Option<PathBuf> {
        GtkGlobalThread::instance().run_blocking(move || run_save_file(build_save_file(&self)))
    }
}

use crate::backend::AsyncFileSaveDialogImpl;
impl AsyncFileSaveDialogImpl for FileDialog {
    fn save_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        async_thread(move || Self::save_file(self).map(FileHandle::wrap))
    }
}

use crate::backend::MessageDialogImpl;
impl MessageDialogImpl for MessageDialog {
    fn show(self) -> MessageDialogResult {
        GtkGlobalThread::instance().run_blocking(move || run_message_dialog(self))
    }
}

use crate::backend::AsyncMessageDialogImpl;
impl AsyncMessageDialogImpl for MessageDialog {
    fn show_async(self) -> DialogFutureType<MessageDialogResult> {
        async_thread(move || Self::show(self))
    }
}
