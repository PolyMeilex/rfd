use std::{
    error::Error, ffi::OsStr, fmt::Display, os::unix::ffi::OsStrExt, path::PathBuf,
    process::Command,
};

use crate::{
    message_dialog::{MessageButtons, MessageLevel},
    FileDialog, MessageDialogResult,
};

#[derive(Debug)]
pub enum ZenityError {
    Io(std::io::Error),
}

impl Error for ZenityError {}

impl Display for ZenityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZenityError::Io(io) => write!(f, "{io}"),
        }
    }
}

impl From<std::io::Error> for ZenityError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub type ZenityResult<T> = Result<T, ZenityError>;

fn command() -> Command {
    let mut cmd = Command::new("zenity");
    cmd.arg("--no-markup");
    cmd
}

fn file_dialog_command(dialog: &FileDialog) -> Command {
    let mut command = command();

    for filter in &dialog.filters {
        command.arg("--file-filter");
        let extensions: Vec<_> = filter
            .extensions
            .iter()
            .map(|ext| {
                if ext == "*" || ext.is_empty() {
                    "*".to_owned()
                } else {
                    format!("*.{}", ext)
                }
            })
            .collect();
        command.arg(format!("{} | {}", filter.name, extensions.join(" ")));
    }

    if let Some(name) = dialog.file_name.as_ref() {
        command.arg("--filename");
        command.arg(name);
    }

    if let Some(dir) = dialog.starting_directory.as_ref() {
        command.current_dir(dir);
    }

    command
}

async fn run(mut command: Command) -> ZenityResult<Option<Vec<u8>>> {
    let res = {
        let (tx, rx) = crate::oneshot::channel();
        std::thread::spawn(move || {
            tx.send(command.output()).ok();
        });
        rx.await.map_err(std::io::Error::other)??
    };
    let buffer = res.stdout;

    Ok((res.status.success() || !buffer.is_empty()).then_some(buffer))
}

/// Strip the single newline zenity prints after its output
fn strip_newline(buffer: &[u8]) -> &[u8] {
    buffer.strip_suffix(b"\n").unwrap_or(buffer)
}

/// Keep the bytes zenity prints as they are, as a file name doesn't have to be UTF-8
fn to_path(bytes: &[u8]) -> PathBuf {
    PathBuf::from(OsStr::from_bytes(bytes))
}

pub async fn pick_file(dialog: &FileDialog) -> ZenityResult<Option<PathBuf>> {
    let mut command = file_dialog_command(dialog);
    command.arg("--file-selection");

    Ok(run(command)
        .await?
        .as_deref()
        .map(strip_newline)
        .map(to_path))
}

pub async fn pick_files(dialog: &FileDialog) -> ZenityResult<Option<Vec<PathBuf>>> {
    let mut command = file_dialog_command(dialog);
    command.args(["--file-selection", "--multiple", "--separator=\n"]);

    Ok(run(command)
        .await?
        .as_deref()
        .map(strip_newline)
        .map(|s| s.split(|&byte| byte == b'\n').map(to_path).collect()))
}

pub async fn pick_folder(dialog: &FileDialog) -> ZenityResult<Option<PathBuf>> {
    let mut command = file_dialog_command(dialog);
    command.args(["--file-selection", "--directory"]);

    Ok(run(command)
        .await?
        .as_deref()
        .map(strip_newline)
        .map(to_path))
}

pub async fn pick_folders(dialog: &FileDialog) -> ZenityResult<Option<Vec<PathBuf>>> {
    let mut command = file_dialog_command(dialog);
    command.args([
        "--file-selection",
        "--directory",
        "--multiple",
        "--separator=\n",
    ]);

    Ok(run(command)
        .await?
        .as_deref()
        .map(strip_newline)
        .map(|s| s.split(|&byte| byte == b'\n').map(to_path).collect()))
}

pub async fn save_file(dialog: &FileDialog) -> ZenityResult<Option<PathBuf>> {
    let mut command = file_dialog_command(dialog);
    command.args(["--file-selection", "--save", "--confirm-overwrite"]);

    Ok(run(command)
        .await?
        .as_deref()
        .map(strip_newline)
        .map(to_path))
}

pub async fn message(
    level: &MessageLevel,
    btns: &MessageButtons,
    title: &str,
    description: &str,
) -> ZenityResult<MessageDialogResult> {
    let cmd = match level {
        MessageLevel::Info => "--info",
        MessageLevel::Warning => "--warning",
        MessageLevel::Error => "--error",
    };

    let ok_label = match btns {
        MessageButtons::Ok => None,
        MessageButtons::OkCustom(ok) => Some(ok),
        _ => None,
    };

    let mut command = command();
    command.args([cmd, "--title", title, "--text", description]);

    if let Some(ok) = ok_label {
        command.args(["--ok-label", ok]);
    }

    run(command).await.map(|res| match res {
        Some(_) => match btns {
            MessageButtons::OkCustom(ok) => MessageDialogResult::Custom(ok.clone()),
            _ => MessageDialogResult::Ok,
        },
        None => MessageDialogResult::Cancel,
    })
}

pub async fn question(
    btns: &MessageButtons,
    title: &str,
    description: &str,
) -> ZenityResult<MessageDialogResult> {
    let mut command = command();
    command.args(["--question", "--title", title, "--text", description]);

    match btns {
        MessageButtons::OkCancel => {
            command.args(["--ok-label", "Ok"]);
            command.args(["--cancel-label", "Cancel"]);
        }
        MessageButtons::OkCancelCustom(ok, cancel) => {
            command.args(["--ok-label", ok.as_str()]);
            command.args(["--cancel-label", cancel.as_str()]);
        }
        MessageButtons::YesNoCancel => {
            command.args(["--extra-button", "No"]);
            command.args(["--cancel-label", "Cancel"]);
        }
        MessageButtons::YesNoCancelCustom(yes, no, cancel) => {
            command.args(["--ok-label", yes.as_str()]);
            command.args(["--cancel-label", cancel.as_str()]);
            command.args(["--extra-button", no.as_str()]);
        }
        _ => {}
    }

    run(command).await.map(|res| match btns {
        MessageButtons::OkCancel => match res {
            Some(_) => MessageDialogResult::Ok,
            None => MessageDialogResult::Cancel,
        },
        MessageButtons::YesNo => match res {
            Some(_) => MessageDialogResult::Yes,
            None => MessageDialogResult::No,
        },
        MessageButtons::OkCancelCustom(ok, cancel) => match res {
            Some(_) => MessageDialogResult::Custom(ok.clone()),
            None => MessageDialogResult::Custom(cancel.clone()),
        },
        MessageButtons::YesNoCancel => match res {
            Some(output) if output.is_empty() => MessageDialogResult::Yes,
            Some(_) => MessageDialogResult::No,
            None => MessageDialogResult::Cancel,
        },
        MessageButtons::YesNoCancelCustom(yes, no, cancel) => match res {
            Some(output) if output.is_empty() => MessageDialogResult::Custom(yes.clone()),
            Some(_) => MessageDialogResult::Custom(no.clone()),
            None => MessageDialogResult::Custom(cancel.clone()),
        },
        _ => MessageDialogResult::Cancel,
    })
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::PathBuf};

    use crate::FileDialog;

    #[test]
    #[ignore]
    fn message() {
        pollster::block_on(super::message(
            &crate::message_dialog::MessageLevel::Info,
            &crate::message_dialog::MessageButtons::Ok,
            "hi",
            "me",
        ))
        .unwrap();
        pollster::block_on(super::message(
            &crate::message_dialog::MessageLevel::Warning,
            &crate::message_dialog::MessageButtons::Ok,
            "hi",
            "me",
        ))
        .unwrap();
        pollster::block_on(super::message(
            &crate::message_dialog::MessageLevel::Error,
            &crate::message_dialog::MessageButtons::Ok,
            "hi",
            "me",
        ))
        .unwrap();
    }

    #[test]
    #[ignore]
    fn question() {
        pollster::block_on(super::question(
            &crate::message_dialog::MessageButtons::OkCancel,
            "hi",
            "me",
        ))
        .unwrap();
        pollster::block_on(super::question(
            &crate::message_dialog::MessageButtons::YesNo,
            "hi",
            "me",
        ))
        .unwrap();
    }

    #[test]
    #[ignore]
    fn pick_file() {
        let path = pollster::block_on(super::pick_file(&FileDialog::default())).unwrap();
        dbg!(path);
    }

    #[test]
    #[ignore]
    fn pick_files() {
        let path = pollster::block_on(super::pick_files(&FileDialog::default())).unwrap();
        dbg!(path);
    }

    #[test]
    #[ignore]
    fn pick_folder() {
        let path = pollster::block_on(super::pick_folder(&FileDialog::default())).unwrap();
        dbg!(path);
    }

    #[test]
    #[ignore]
    fn save_file() {
        let path = pollster::block_on(super::save_file(&FileDialog::default())).unwrap();
        dbg!(path);
    }

    #[test]
    fn output_path() {
        assert_eq!(
            super::to_path(super::strip_newline(b"/home/user/caf\xe9.txt\n")),
            PathBuf::from(OsStr::from_bytes(b"/home/user/caf\xe9.txt"))
        );
        assert_eq!(super::strip_newline(b"x\n\n"), b"x\n");
    }
}
